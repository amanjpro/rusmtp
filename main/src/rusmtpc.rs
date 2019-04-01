pub mod clients;

extern crate protocol;

#[macro_use]
extern crate log;

extern crate rand;
extern crate fs2;
extern crate common;

extern crate log4rs;
extern crate dirs;

use std::alloc::System;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::{thread, time};
use std::io::{self, Read, Write};
use rand::random;
use fs2::FileExt;
use dirs::home_dir;
use crate::clients::send_to_daemon;
use common::*;
use common::args::*;
use common::mail::*;
use common::config::*;

#[global_allocator]
static GLOBAL: System = System;

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

    let retry = args.flag_with_retry.unwrap_or(false);
    let spool_root = &conf.spool_root;

    let lock_file = File::open(&flock_path).unwrap_or_else(|_| {
        enqueue(&mail, spool_root, retry);
        log_and_panic(&format!("Cannot open flock {}", flock_path))
    });
    let ten_millis = time::Duration::from_millis(10);
    while lock_file.lock_exclusive().is_err() {
        thread::sleep(ten_millis);
    }

    let res = send_to_daemon(&mail, &conf.socket_root, conf.timeout, account);
    if res.is_err() {
        enqueue(&mail, spool_root, retry);
        let _: String = log_and_panic(&res.unwrap_err());
    }
    let _ = lock_file.unlock();
}

fn enqueue(mail: &Mail, spool_root: &str, should_retry: bool) {
    if should_retry {
        let account = &mail.account.as_ref().unwrap();
        let rand: u64 = random::<u64>();;
        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let mut email_file = File::create(
            format!("{}/{}-{}-{}", spool_root,
                    &account, rand, since_the_epoch.as_secs()))
            .expect("Cannot archive the email, failing...");
        email_file.write_all(mail.serialize().as_slice())
            .expect("Cannot archive the email, failing...");
    }
}
