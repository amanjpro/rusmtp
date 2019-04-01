use protocol::{Raven, Authentication};
use common::{ERROR_SIGNAL,OK_SIGNAL,get_socket_path};
use common::mail::Mail;
use common::vault::Vault;
use common::account::Account;
use native_tls::TlsStream;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::ops::Deref;
use std::net::TcpStream;

pub struct DefaultClient {
    pub account: Account,
}

impl DefaultClient {
    fn get_mailer<R: Raven>(&self, vault: &Vault, passwd: &[u8]) -> Option<R> {
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

        let cert_root = account.cert_root.as_ref().map(|c| c.to_owned());

        let timeout = account.timeout;

        let mailer = R::create_connection(&host, *port, timeout, cert_root);

        let mut mailer = match mailer {
            Ok(mailer) => mailer,
            Err(error) => {
                error!("{}", error);
                return None;
            }
        };

        match mailer.hand_shake(host) {
            Ok(auths) => {
                if auths.contains(&Authentication::Login) {
                    if let Err(error) = mailer.authenticate_with_login(&username.as_ref(),
                        &vault.decrypt(passwd).into_bytes()) {
                        error!("{}", error);
                        return None;
                    };
                }
                Some(mailer)
            },
            Err(error) => {
                error!("{}", error);
                None
            }
        }
    }

    fn send_email<R: Raven>(&self, stream: &mut UnixStream, vault: &Vault) {
            //where I = Incoming + Write + Read {
        let account = &self.account;
        let password = account.password.as_ref().unwrap();
        let label = &account.label;
        let mailer = self.get_mailer::<R>(vault, &password);
        if mailer.is_none() {
            error!("Cannot open a connection for account {}", label);
            let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
            return;
        }

        let mut mailer = mailer.unwrap();

        if account.username.is_none() {
            error!("Please configure the username for {}", label);
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
        mailer.close();
    }

    pub fn start(&self, prefix: &str, vault: &Vault) {
        let account = &self.account;

        let label = &account.label;

        let tls = &account.tls.unwrap_or(false);
        if account.password.is_none() {
            error!("Password is not defined for {}", &label);
            return;
        }

        if let Ok(listener) = UnixListener::bind(get_socket_path(prefix, &label)) {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        if *tls {
                            self.send_email::<TlsStream<TcpStream>>(&mut stream, vault);
                        } else {
                            self.send_email::<TcpStream>(&mut stream, vault);
                        }
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


