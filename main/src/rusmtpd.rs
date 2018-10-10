extern crate serde_json;
extern crate common;
extern crate secstr;
extern crate esmtp_client;

use secstr::SecStr;
use common::*;
use esmtp_client::SMTPConnection;

use std::os::unix::net::{UnixStream, UnixListener};
use std::process::{Command, Stdio};
use std::io::{Read, Write};
use std::error::Error;
use std::{str,fs};
use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc};
use std::ops::Deref;



fn send_mail_with_external_client(mut stream: UnixStream, client: &str, passwd: &SecStr) {
    let mut mail = String::new();
    let _ = stream.read_to_string(&mut mail).unwrap();
    let mail: Mail = serde_json::from_str(&mail).expect("Cannot parse the mail");
    let recipients: Vec<String> = mail.recipients;
    let body = mail.body;

    let smtp = Command::new(client)
      .arg(format!("--passwordeval=echo {}", str::from_utf8(passwd.unsecure()).unwrap()))
      .args(recipients)
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .spawn()
      .expect("Failed to start smtp process");

    match smtp.stdin.unwrap().write_all(body.as_slice()) {
        Err(why) => {
            let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
            panic!("couldn't write to smtp stdin: {}", why.description());
        },
        Ok(_) => {
            let _ = stream.write_all(OK_SIGNAL.as_bytes());
            println!("email sent to smtp");
        },
    }
}

fn external_smtp_client(client: &str, label: &str, passwd: &SecStr) {
    if let Ok(listener) = UnixListener::bind(get_socket_path(&label)) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                  send_mail_with_external_client(stream, &client, &passwd);
                }
                _              => {
                    /* connection failed */
                    break;
                }
            }
        }
    } else {
        panic!("failed to open a socket")
    }
}

fn default_smtp_client(account: Account, passwd: &mut SecStr) {
    let label    = account.label;

    let host     = account.host
        .expect(&format!("Please configure the host for {}", label));

    let username = account.username
        .expect(&format!("Please configure the username for {}", label));
    let port     = account.port
        .expect(&format!("Please configure the port for {}", label));

    let mut mailer = SMTPConnection::open_connection(&host, port);

    if mailer.supports_login {
        mailer.login(&SecStr::from(username.clone()), &passwd);
    }

    let mailer = Arc::new(Mutex::new(mailer));

    {
        let mailer = mailer.clone();
        let heartbeat = account.heartbeat as u64;
        thread::spawn(move || {
            let sleep_time = Duration::from_secs(heartbeat * 60);
            loop {
                mailer.lock().expect("Cannot get the mailer instance to keep it alive")
                    .keep_alive(); thread::sleep(sleep_time)
            }
        });
    }
    passwd.zero_out();
    if let Ok(listener) = UnixListener::bind(get_socket_path(&label)) {

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut mail = String::new();
                    stream.read_to_string(&mut mail).unwrap();
                    let mail: Mail = serde_json::from_str(&mail).expect("Cannot parse the mail");
                    let recipients: Vec<&str> = mail.recipients.iter().filter(|&s| s != "--").map(|s| s.deref()).collect();
                    let body = mail.body;
                    mailer.lock().expect("Cannot get the mailer instance to send an email")
                        .send_mail(&username, &recipients, &body);
                    let _ = stream.write_all(OK_SIGNAL.as_bytes());
                }
                _          => {
                    /* connection failed */
                    break;
                }
            }
        }
    } else {
        panic!("failed to open a socket")
    }
}

fn start_daemon(conf: Configuration) {
    let mut children = vec![];
    for account in conf.accounts {
        let client = conf.smtpclient.clone();
        children.push(thread::spawn(move || {
            let eval = account.passwordeval.clone();

            if let Ok(result) = Command::new("sh").arg("-c").arg(eval).stdout(Stdio::piped()).spawn() {
                let mut child_stdout = result.stdout.expect("Cannot get the handle of the child process");
                let mut output = String::new();
                let _ = child_stdout.read_to_string(&mut output);

                let mut passwd = SecStr::from(output.trim());
                output.clear();

                // close the socket, if it exists
                let _ = fs::remove_file(get_socket_path(&account.label));

                match client {
                    Some(client) =>
                        external_smtp_client(&client, &account.label, &passwd),
                    None         =>
                        default_smtp_client(account, &mut passwd),
                }
            }
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
}

fn main() {

    let args = process_args("rusmtpd", &rusmtpd_usage("rusmtpd"));
    let conf = read_config(&args.flag_rusmtprc);

    start_daemon(conf);
}

