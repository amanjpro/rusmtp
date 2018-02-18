extern crate serde;
extern crate serde_json;
extern crate common;


use std::os::unix::net::UnixStream;
use std::process::exit;
use std::time::Duration;
use common::{SOCKET_PATH, ERROR_SIGNAL, OK_SIGNAL, Mail};
use std::env;
use std::io::{self, Read, Write};

fn main () {
    let mut full_args: Vec<String> = env::args().collect();
    full_args.remove(0);

    let mut body: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut body).expect("Reading mail from the stdin");

    let mail = Mail {
        recipients: full_args,
        body: body,

    };

    let msg =serde_json::to_string(&mail)
        .expect("Cannot generate JSON for the given message");

    let mut stream = UnixStream::connect(SOCKET_PATH).expect("The daemon is not running, please start it.");
    let timeout = Duration::new(5, 0);
    let _ = stream.set_read_timeout(Some(timeout));
    stream.write_all(msg.as_bytes()).unwrap();
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    if &response == ERROR_SIGNAL { exit(1); }
    else if &response == OK_SIGNAL { exit(0); }
    else {
        println!("Unexpected response from the server: {}", response);
        exit(1);
    }
}
