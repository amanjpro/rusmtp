extern crate serde;
extern crate serde_json;
extern crate common;
extern crate docopt;

#[macro_use]
extern crate serde_derive;

use common::{SOCKET_PATH, Mail};
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::{Read, Write};
use std::error::Error;
use std::fs;

use std::process;
use std::process::{Command, Stdio};
use docopt::Docopt;


// Define the struct that results from those options.
#[derive(Deserialize, Debug)]
struct Args {
    arg_CMD: Option<String>,
    flag_help: bool,
    flag_version: bool,
}

fn main() {

    let APP_VERSION = "1.0.0";
    let APP_NAME = "smtpd";
    // Define a USAGE string.
    let USAGE = format!("
    {}

    Usage: {} --passwordeval CMD
           {} --help
           {} --version

    Options:
        -h, --help               Show this help.
        -v, --version            Show the version.
    ", APP_NAME, APP_NAME, APP_NAME, APP_NAME);

    let args: Args = Docopt::new(USAGE.clone())
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}, v {}", APP_NAME, APP_VERSION);
        process::exit(0);
    }

    if args.flag_help {
        println!("{}", USAGE);
        process::exit(0);
    }

    if let Some(arg_CMD) = args.arg_CMD {
        if let Ok(result) = Command::new("sh").arg("-c").arg(arg_CMD).stdout(Stdio::piped()).spawn() {
            let mut child_stdout = result.stdout.expect("Cannot get the handle of the child process");
            let mut output = String::new();
            child_stdout.read_to_string(&mut output);

            let passwd = output.trim();
            println!("{}", passwd);

            // close the socket, if it exists
            fs::remove_file(SOCKET_PATH);

            if let Ok(listener) = UnixListener::bind(SOCKET_PATH) {

                // accept connections and process them, spawning a new thread for each one
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                          let mut mail = String::new();
                          stream.read_to_string(&mut mail).unwrap();
                          println!("{}", mail);
                          let mail: Mail = serde_json::from_str(&mail).expect("Cannot parse the mail");
                          let recipients: Vec<String> = mail.recipients;
                          let body = mail.body;

                          let msmtp = Command::new("msmtp").arg(format!("msmtp --passwordeval=\"echo {}\"", passwd))
                              .args(recipients)
                              .stdin(Stdio::piped())
                              .stdout(Stdio::null())
                              .spawn()
                              .expect("Failed to start msmtp process");

                          match msmtp.stdin.unwrap().write_all(body.as_slice()) {
                             Err(why) => panic!("couldn't write to msmtp stdin: {}", why.description()),
                             Ok(_) => println!("email sent to msmtp"),
                          }

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
    } else {
        println!("{}", USAGE);
        process::exit(1);
    }
}
