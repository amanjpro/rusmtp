extern crate serde;
extern crate serde_json;
extern crate common;
extern crate docopt;
extern crate secstr;

#[macro_use]
extern crate serde_derive;

use secstr::SecStr;
use common::{SOCKET_PATH, Mail, Configuration, read_config};

use std::os::unix::net::{UnixStream, UnixListener};
use std::process::{exit, Command, Stdio};
use std::io::{Read, Write};
use std::env::home_dir;
use std::error::Error;
use std::{str,fs};

use docopt::Docopt;


// Define the struct that results from those options.
#[derive(Deserialize, Debug)]
struct Args {
    flag_smtpdrc: String,
    flag_help: bool,
    flag_version: bool,
}

fn send_mail(mut stream: UnixStream, client: &str, passwd: &SecStr) {
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
        Err(why) => panic!("couldn't write to smtp stdin: {}", why.description()),
        Ok(_) => println!("email sent to smtp"),
    }
}

fn start_daemon(conf: Configuration) {
    let eval = &conf.passwordeval;
    let client = conf.smtpclient;

    if let Ok(result) = Command::new("sh").arg("-c").arg(eval).stdout(Stdio::piped()).spawn() {
        let mut child_stdout = result.stdout.expect("Cannot get the handle of the child process");
        let mut output = String::new();
        child_stdout.read_to_string(&mut output);

        let passwd = SecStr::from(output.trim());

        // close the socket, if it exists
        fs::remove_file(SOCKET_PATH);

        match client {

            Some(client) =>
                if let Ok(listener) = UnixListener::bind(SOCKET_PATH) {

                    for stream in listener.incoming() {
                        match stream {
                            Ok(mut stream) => {
                              send_mail(stream, &client, &passwd);
                            }
                            Err(err) => {
                                /* connection failed */
                                break;
                            }
                        }
                    }
                } else {
                    panic!("failed to open a socket")
                },
            None         => ()
        }
    }
}

fn main() {

    let home_dir = home_dir().expect("Cannot find the home directory");
    let home_dir = home_dir.display();

    let APP_VERSION = env!("CARGO_PKG_VERSION");
    let APP_NAME = "smtpd";
    // Define a USAGE string.
    let USAGE = format!("
    {}

    Usage: {0}
           {0} --smtpdrc=<string>
           {0} --help
           {0} --version

    Options:
        --smtpdrc=<string>       Path to the smtpdrc [default: {}/.smtpdrc]
        -h, --help               Show this help.
        -v, --version            Show the version.
    ", APP_NAME, home_dir);

    let args: Args = Docopt::new(USAGE.clone())
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}, v {}", APP_NAME, APP_VERSION);
        exit(0);
    }

    if args.flag_help {
        println!("{}", USAGE);
        exit(0);
    }

    let conf = read_config(&args.flag_smtpdrc);

    start_daemon(conf);
}

