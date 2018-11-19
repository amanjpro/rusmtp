pub mod clients;

extern crate common;
extern crate protocol;
extern crate fs2;

#[macro_use]
extern crate log;
extern crate dirs;

use std::alloc::System;
use std::process::{Command, Stdio};
use std::fs::{remove_file, File};
use std::io::{Read, Error};
use std::path::Path;
use dirs::home_dir;
use fs2::FileExt;
use std::{thread, time, fs, thread::JoinHandle};
use common::*;
use common::args::*;
use common::mail::*;
use common::config::*;
use common::account::*;
use clients::*;
use clients::external::*;
use clients::default::*;

#[global_allocator]
static GLOBAL: System = System;

fn print_welcome_message() {
    println!("rusmtpd daemon started...");
    println!("Ready to send emails");
}

fn retry_logic(spool_path: &str, flock_root: &str, socket_root: &str,
               timeout: u64) -> Result<(), Error> {
    let spool_dir = Path::new(&spool_path);
    if spool_dir.is_dir() {
        let spool_dir = fs::read_dir(spool_dir)?;
        for entry in spool_dir {
            let path = entry?.path();
            if path.is_file() {
                if let Some(path_string) =
                    path.clone().file_name().and_then(|f| f.to_str()) {
                    let v: Vec<&str> = path_string.split('-').collect();
                    if v.len() == 3 {
                        let account = v[0];
                        let flock_path = get_lock_path(&flock_root, account);
                        let lock_file = File::open(&flock_path)?;
                        // It is safe to assume that everything is written
                        // for this account contains the complete email message,
                        // because this thread already acquires the flock,
                        // which is only available if there is no active writes
                        // to the emails of this account.
                        lock_file.lock_exclusive()?;
                        let file = &mut File::open(path.clone())?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        if let Ok(mail) = Mail::deserialize(&mut contents) {
                            if send_to_daemon(&mail, &socket_root, timeout, account).is_ok() {
                                remove_file(path)?;
                            }
                        }
                        let _ = lock_file.unlock();
                    }
                }
            }
        }
    }
    Ok(())
}

fn start_resender(spool_path: String, flock_root: String,
                  socket_root: String, timeout: u64) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let _ = retry_logic(&spool_path, &flock_root, &socket_root, timeout);
            let one_minute = time::Duration::new(60, 0);
            thread::sleep(one_minute);
        }
    })
}

fn start_daemon(conf: Configuration) -> Vec<JoinHandle<()>> {
    let mut children = vec![];
    for account in conf.accounts {
        let client = conf.smtpclient.clone();
        let socket_root = conf.socket_root.clone();
        children.push(thread::spawn(move || {
            let eval = account.passwordeval.clone();

            if let Ok(result) = Command::new("sh").arg("-c")
                    .arg(eval).stdout(Stdio::piped()).spawn() {
                let child_stdout = result.stdout;
                if child_stdout.is_none() {
                    error!("Cannot get the handle of the child process");
                } else {
                    let mut child_stdout = child_stdout.unwrap();
                    let mut passwd = String::new();
                    let _ = child_stdout.read_to_string(&mut passwd);


                    // close the socket, if it exists
                    let _ = fs::remove_file(get_socket_path(&socket_root, &account.label));

                    let account = Account {
                        label: account.label,
                        username: account.username,
                        passwordeval: account.passwordeval,
                        host: account.host,
                        port: account.port,
                        tls: account.tls,
                        default: account.default,
                        password: Some(account.vault.encrypt(&mut passwd)),
                        vault: account.vault,
                    };

                    match client {
                        Some(client) => {
                            let external_client = ExternalClient::new(&client);
                            external_client.start(&account.label,
                                                  &socket_root,
                                                  &account.vault,
                                                  &passwd.into_bytes());
                        },
                        None         => {
                            let default_client = DefaultClient::new(account);
                            default_client.start(&socket_root, &default_client.account.vault);

                        },
                    }
                }
            }
        }));
    }

    children
}

fn main() {
    log4rs::init_file(format!("{}/.rusmtp/rusmtpd-log4rs.yaml",
          home_dir().expect("Cannot find the home directory").display()),
          Default::default()).unwrap();

    let args = process_args("rusmtpd", &rusmtpd_usage("rusmtpd"));
    let conf = read_config(&args.flag_rusmtprc);

    info!("rusmtpd started");

    print_welcome_message();
    let resender = start_resender(conf.spool_root.clone(),
                                  conf.flock_root.clone(),
                                  conf.socket_root.clone(), conf.timeout);
    let senders = start_daemon(conf);

    let _ = resender.join();
    for sender in senders {
        // Wait for the thread to finish. Returns a result.
        let _ = sender.join();
    }
}
