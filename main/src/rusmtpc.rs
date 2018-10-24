extern crate serde_json;
extern crate common;


use std::os::unix::net::UnixStream;
use std::process::exit;
use std::net::Shutdown;
use std::time::Duration;
use common::*;
use common::args::*;
use common::mail::*;
use common::config::*;
use std::io::{self, Read, Write};

fn main () {

    let args = process_args("rusmtpc", &rusmtpc_usage("rusmtpc"));
    let conf = read_config(&args.flag_rusmtprc);


    let mut body: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut body).expect("Reading mail from the stdin");

    let mail = Mail {
        recipients: args.arg_recipients,
        body,
        account: args.flag_account,
    };

    let msg =serde_json::to_string(&mail)
        .expect("Cannot generate JSON for the given message");

    let account = &mail.account.unwrap_or({
      let &value = conf.accounts.iter()
        .filter(|acc| acc.default)
        .map(|x| &x.label)
        .collect::<Vec<_>>()
        .first()
        .expect("Please pass a valid account name or set a default account");
      value.to_string()
    });

    let mut stream = UnixStream::connect(get_socket_path(account))
        .expect("The daemon is not running, please start it.");
    stream.write_all(msg.as_bytes()).unwrap();
    let _ = stream.shutdown(Shutdown::Write);
    let timeout = Duration::new(conf.timeout, 0);
    let _ = stream.set_read_timeout(Some(timeout));
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response).expect("Timeout is met, please retry");
    let response = String::from_utf8(response).expect("Cannot decode the response");
    if ERROR_SIGNAL == response { exit(1); }
    else if OK_SIGNAL == response { exit(0); }
    else {
        println!("Unexpected response from the server: {}", response);
        exit(1);
    }
}
