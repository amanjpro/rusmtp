extern crate serde;
extern crate serde_json;
extern crate common;


use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use common::{SOCKET_PATH, Mail};
use std::env;
use std::io::{self, Read};

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
    stream.write_all(msg.as_bytes()).unwrap();
}
