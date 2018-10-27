use protocol::SMTPConnection;
use {OK_SIGNAL,get_socket_path};
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
    fn get_mailer(&self, vault: &Vault, passwd: &[u8]) -> SMTPConnection {
        let account = &self.account;

        let label    = &account.label.to_string();

        let host     = &account.host
            .as_ref()
            .unwrap_or_else(|| panic!("Please configure the host for {}", label));

        let username = account.username
            .as_ref()
            .unwrap_or_else(|| panic!("Please configure the username for {}", label));

        let port     = account.port
            .as_ref()
            .unwrap_or_else(|| panic!("Please configure the port for {}", label));

        let mut mailer = SMTPConnection::open_connection(&host, *port);

        if mailer.supports_login {
            mailer.login(&username.clone().into_bytes(),
                &vault.decrypt(passwd).into_bytes());
        }

        mailer
    }

    pub fn start(&self, prefix: &str, vault: &Vault) {
        let account = &self.account;

        let label = &account.label;

        let password = account.password.clone()
            .unwrap_or_else(|| panic!("Password is not defined for {}", &label));

        if let Ok(listener) = UnixListener::bind(get_socket_path(prefix, &label)) {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut mailer = self.get_mailer(vault, &password);

                        let username = &account.username
                            .as_ref()
                            .unwrap_or_else(|| panic!("Please configure the username for {}", &label));
    println!("HERE");
                        let mut mail = String::new();
                        stream.read_to_string(&mut mail).unwrap();
                        // TODO: Failure here? should be reported back to rusmtpc
                        let mail = Mail::deserialize(&mut mail.into_bytes()).unwrap();
                        let recipients: Vec<&str> = mail.recipients.iter().filter(|&s| s != "--").map(|s| s.deref()).collect();
                        let body = mail.body;
                        mailer.send_mail(&username, &recipients, &body);
                        let _ = stream.write_all(OK_SIGNAL.as_bytes());
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


