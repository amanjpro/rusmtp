extern crate fs2;
extern crate common;

extern crate log4rs;
extern crate dirs;

use std::fs::File;
use std::path::Path;
use std::{thread, time};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::process::exit;
use std::net::Shutdown;
use std::time::Duration;
use fs2::FileExt;
use dirs::home_dir;
use common::*;
use common::args::*;
use common::mail::*;
use common::config::*;

fn main () {
    log4rs::init_file(format!("{}/.rusmtp/rusmtpc-log4rs.yaml",
          home_dir().expect("Cannot find the home directory").display()),
          Default::default()).unwrap();

    let args = process_args("rusmtpc", &rusmtpc_usage("rusmtpc"));
    let conf = read_config(&args.flag_rusmtprc);


    let mut body: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut body).unwrap_or_else(|_|
        log_and_panic("Reading mail from the stdin"));

    let mail = Mail {
        recipients: args.arg_recipients,
        body,
        account: args.flag_account,
    };

    let account = &mail.account.as_ref().unwrap_or_else(|| {
      let &value = conf.accounts.iter()
        .filter(|acc| acc.default)
        .map(|x| &x.label)
        .collect::<Vec<_>>()
        .first()
        .unwrap_or_else(||
            log_and_panic("Please pass a valid account name or set a default account"));
      value
    });

    let flock_path = get_lock_path(&conf.flock_root, account);

    if ! Path::new(&flock_path).exists() {
        let _ = File::create(&flock_path);
    }

    let lock_file = File::open(&flock_path).unwrap_or_else(|_|
        log_and_panic(&format!("Cannot open flock {}", flock_path)));
    let ten_millis = time::Duration::from_millis(10);
    while lock_file.lock_exclusive().is_err() {
        thread::sleep(ten_millis);
    }

    let socket_path = get_socket_path(&conf.socket_root, account);
    let mut stream = UnixStream::connect(socket_path)
        .unwrap_or_else(|_|
            log_and_panic("The daemon is not running, please start it."));
    stream.write_all(mail.serialize().as_slice())
        .unwrap_or_else(|_| log_and_panic("Cannot write email to the Unix socket"));
    let _ = stream.shutdown(Shutdown::Write);
    let timeout = Duration::new(conf.timeout, 0);
    let _ = stream.set_read_timeout(Some(timeout));
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response).unwrap_or_else(|_|
        log_and_panic("Timeout is met, please retry"));
    let response = String::from_utf8(response).unwrap_or_else(|_|
        log_and_panic("Cannot decode the response"));
    let _ = lock_file.unlock();
    if ERROR_SIGNAL == response { exit(1); }
    else if OK_SIGNAL == response { exit(0); }
    else {
        log_and_panic(&format!("Unexpected response from the server: {}", response))
    }
}
