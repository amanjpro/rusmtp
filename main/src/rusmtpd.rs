extern crate common;
extern crate protocol;

#[macro_use]
extern crate log;
extern crate dirs;

pub mod clients;

use std::process::{Command, Stdio};
use std::io::Read;
use dirs::home_dir;
use std::{thread,fs};
use common::*;
use common::args::*;
use common::config::*;
use common::account::*;
use clients::external::*;
use clients::default::*;


fn print_welcome_message() {
    println!("rusmtpd daemon started...");
    println!("Ready to send emails");
}

fn start_daemon(conf: Configuration) {
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

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
}

fn main() {
    log4rs::init_file(format!("{}/log4rs.yaml",
          home_dir().expect("Cannot find the home directory").display()),
          Default::default()).unwrap();

    let args = process_args("rusmtpd", &rusmtpd_usage("rusmtpd"));
    let conf = read_config(&args.flag_rusmtprc);

    info!("rusmtpd started");

    print_welcome_message();
    start_daemon(conf);
}
