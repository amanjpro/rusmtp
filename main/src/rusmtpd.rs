extern crate common;

use common::*;
use common::args::*;
use common::config::*;
use common::account::*;
use common::clients::external::*;
use common::clients::default::*;
use std::process::{Command, Stdio};
use std::io::Read;
use std::{thread,fs};

fn start_daemon(conf: Configuration) {
    let mut children = vec![];
    for account in conf.accounts {
        let client = conf.smtpclient.clone();
        children.push(thread::spawn(move || {
            let eval = account.passwordeval.clone();

            if let Ok(result) = Command::new("sh").arg("-c").arg(eval).stdout(Stdio::piped()).spawn() {
                let mut child_stdout = result.stdout.expect("Cannot get the handle of the child process");
                let mut passwd = String::new();
                let _ = child_stdout.read_to_string(&mut passwd);


                // close the socket, if it exists
                let _ = fs::remove_file(get_socket_path(&account.label));

                let account = if account.mode == AccountMode::Secure {
                    Account {
                        label: account.label,
                        username: account.username,
                        passwordeval: account.passwordeval,
                        mode: account.mode,
                        host: account.host,
                        port: account.port,
                        tls: account.tls,
                        heartbeat: account.heartbeat,
                        default: account.default,
                        password: Some(account.vault.encrypt(&mut passwd)),
                        vault: account.vault,
                    }
                } else {
                    account
                };

                match client {
                    Some(client) => {
                        let external_client = ExternalClient::new(&client);
                        external_client.start(&account.label,
                                              &account.vault, 
                                              &passwd.into_bytes());
                    },
                    None         => {
                        let default_client = DefaultClient::new(account);
                        default_client.start(&default_client.account.vault);
                    },
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

    let args = process_args("rusmtpd", &rusmtpd_usage("rusmtpd"));
    let conf = read_config(&args.flag_rusmtprc);

    start_daemon(conf);
}

