use protocol::SMTPConnection;
use {OK_SIGNAL,get_socket_path};
use mail::Mail;
use vault::Vault;
use account::{Account, AccountMode};
use std::os::unix::net::UnixListener;
use std::net::Shutdown;
use std::time::Duration;
use std::sync::{Mutex, Arc};
use std::io::{Read, Write};
use std::thread;
use std::ops::Deref;
use serde_json;

pub struct DefaultClient {
    pub account: Account,
}

impl DefaultClient {
    fn get_mailer(&self, vault: &Vault, passwd: &[u8]) -> Arc<Mutex<SMTPConnection>> {
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

        Arc::new(Mutex::new(mailer))
    }

    fn maintain_connection(&self, mailer: Arc<Mutex<SMTPConnection>>, heartbeat: u64) {
        thread::spawn(move || {
            let sleep_time = Duration::from_secs(heartbeat * 60);
            loop {
                mailer.lock().expect("Cannot get the mailer instance to keep it alive")
                    .keep_alive(); thread::sleep(sleep_time)
            }
        });
    }

    pub fn start(&self, vault: &Vault) {
        let account = &self.account;

        let label = &account.label;

        let password = account.password.clone()
            .unwrap_or_else(|| panic!("Password is not defined for {}", &label));

        let mailer = self.get_mailer(vault, &password);

        if let AccountMode::Paranoid = &account.mode {
            let mailer = mailer.clone();
            let heartbeat = &account.heartbeat;
            self.maintain_connection(mailer, *heartbeat);
        }

        if let Ok(listener) = UnixListener::bind(get_socket_path(&label)) {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mailer = if account.mode == AccountMode::Secure {
                            self.get_mailer(vault, &password)
                        } else { mailer.clone() };

                        let username = &account.username
                            .as_ref()
                            .unwrap_or_else(|| panic!("Please configure the username for {}", &label));
                        let mut mail = String::new();
                        stream.read_to_string(&mut mail).unwrap();
                        let mail: Mail = serde_json::from_str(&mail).expect("Cannot parse the mail");
                        let recipients: Vec<&str> = mail.recipients.iter().filter(|&s| s != "--").map(|s| s.deref()).collect();
                        let body = mail.body;
                        mailer.lock().expect("Cannot get the mailer instance to send an email")
                            .send_mail(&username, &recipients, &body);
                        let _ = stream.write_all(OK_SIGNAL.as_bytes());
                        if account.mode == AccountMode::Secure {
                            stream.shutdown(Shutdown::Both).expect("shutdown function failed");
                        }

                    }
                    _          => {
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


