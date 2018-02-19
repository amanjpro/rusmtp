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
    stream.read_to_string(&mut mail).unwrap();
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
            stream.write_all(ERROR_SIGNAL.as_bytes());
            panic!("couldn't write to smtp stdin: {}", why.description());
        },
        Ok(_) => {
            stream.write_all(OK_SIGNAL.as_bytes());
            println!("email sent to smtp");
        },
    }
}

fn external_smtp_client(client: &str, passwd: &SecStr) {
    if let Ok(listener) = UnixListener::bind(SOCKET_PATH) {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                  send_mail_with_external_client(stream, &client, &passwd);
                }
                Err(err) => {
                    /* connection failed */
                    break;
                }
            }
        }
    } else {
        panic!("failed to open a socket")
    }
}

fn default_smtp_client(conf: Configuration, passwd: &mut SecStr) {
    let host     = conf.host.expect("Please configure the SMTP host");
    let username = conf.username.expect("Please configure the username");
    let port     = conf.port.expect("Please configure the port");

    let mut mailer = SMTPConnection::open_connection(&host, port);

    if mailer.supports_login {
        mailer.login(&SecStr::from(username.clone()), &passwd);
    }

    let mut mailer = Arc::new(Mutex::new(mailer));

    {
        let mailer = mailer.clone();
        let heartbeat = conf.heartbeat as u64;
        thread::spawn(move || {
            let sleep_time = Duration::from_secs(heartbeat * 60);
            loop {
                mailer.lock().expect("Cannot get the mailer instance to keep it alive")
                    .keep_alive(); thread::sleep(sleep_time)
            }
        });
    }
    passwd.zero_out();
    if let Ok(listener) = UnixListener::bind(SOCKET_PATH) {

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
                    stream.write_all(OK_SIGNAL.as_bytes());
                }
                Err(err) => {
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
    let eval = &conf.passwordeval.clone();
    let client = conf.smtpclient.clone();

    if let Ok(result) = Command::new("sh").arg("-c").arg(eval).stdout(Stdio::piped()).spawn() {
        let mut child_stdout = result.stdout.expect("Cannot get the handle of the child process");
        let mut output = String::new();
        child_stdout.read_to_string(&mut output);

        let mut passwd = SecStr::from(output.trim());
        output.clear();

        // close the socket, if it exists
        fs::remove_file(SOCKET_PATH);

        match client {
            Some(client) =>
                external_smtp_client(&client, &passwd),
            None         =>
                default_smtp_client(conf, &mut passwd),
        }
    }
}

fn main() {

    let args = process_args("smtpd", &smtpd_usage("smptd"));
    let conf = read_config(&args.flag_smtpdrc);

    start_daemon(conf);
}

