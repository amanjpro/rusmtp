use protocol::SMTPConnection;
use common::{ERROR_SIGNAL,OK_SIGNAL,get_socket_path};
use mail::Mail;
use vault::Vault;
use account::Account;
use std::os::unix::net::UnixListener;
use std::io::{Read, Write};
use std::ops::Deref;

pub struct DefaultClient {
    pub account: Account,
}

impl DefaultClient {
    fn get_mailer(&self, vault: &Vault, passwd: &[u8]) -> Option<SMTPConnection> {
        let account = &self.account;

        let label    = &account.label.to_string();

        if account.host.is_none() {
            error!("Please configure the host for {}", label);
            return None;
        } else if account.username.is_none() {
            error!("Please configure the username for {}", label);
            return None;
        } else if account.port.is_none() {
            error!("Please configure the port for {}", label);
            return None;
        }

        let host     = account.host.as_ref().unwrap();

        let username = account.username.as_ref().unwrap();

        let port     = account.port.as_ref().unwrap();

        let mut mailer = match SMTPConnection::open_connection(&host, *port) {
            Ok(mailer) => mailer,
            Err(error) => {
                error!("{}", error);
                return None;
            }
        };

        if mailer.supports_login {
            if let Err(error) = mailer.login(&username.as_ref(),
                &vault.decrypt(passwd).into_bytes()) {
                error!("{}", error);
                return None;
            };

        }

        Some(mailer)
    }

    pub fn start(&self, prefix: &str, vault: &Vault) {
        let account = &self.account;

        let label = &account.label;

        if account.password.is_none() {
            error!("Password is not defined for {}", &label);
            return;
        }
        let password = account.password.as_ref().unwrap();

        if let Ok(listener) = UnixListener::bind(get_socket_path(prefix, &label)) {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mailer = self.get_mailer(vault, &password);
                        if mailer.is_none() {
                            error!("Cannot open a connection for account {}", &label);
                            let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                            return;
                        }

                        let mut mailer = mailer.unwrap();

                        if account.username.is_none() {
                            error!("Please configure the username for {}", &label);
                            let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                            return;
                        }

                        let username = &account.username.as_ref().unwrap();
                        let mut mail = Vec::new();

                        if let Err(e) = stream.read_to_end(&mut mail) {
                            error!("Error happened while reading the incoming email {}", e);
                            let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                        }

                        let mail = Mail::deserialize(&mut mail);
                        match mail {
                            Ok(mail)  => {
                                let recipients: Vec<&str> = mail.recipients.iter()
                                    .filter(|&s| s != "--").map(|s| s.deref()).collect();
                                let body = mail.body;
                                if let Err(error) = mailer.send_mail(&username, &recipients, &body) {
                                    error!("{}", error);
                                    let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                                } else {
                                    let _ = stream.write_all(OK_SIGNAL.as_bytes());
                                }
                            },
                            Err(e) => {
                                error!("Error happened while reading the incoming email {}", e);
                                let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                            }
                        }
                        mailer.shutdown();

                    }
                    _                 => {
                        /* connection failed */
                        break;
                    }
                }
            }
        } else {
            panic!("failed to open a socket")
        }
    }

    pub fn new(account: Account) -> Self {
        DefaultClient { account }
    }
}


