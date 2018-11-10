use common::{OK_SIGNAL,ERROR_SIGNAL,get_socket_path};
use mail::Mail;
use vault::Vault;
use std::os::unix::net::{UnixStream, UnixListener};
use std::process::{Command, Stdio};
use std::str;
use std::io::{Read, Write};
use std::error::Error;

pub struct ExternalClient {
    pub client: String,
}

impl ExternalClient {
    fn send_mail(&self, mut stream: UnixStream, passwd: &[u8]) {
        let mut mail = Vec::new();
        if let Err(e) = stream.read_to_end(&mut mail) {
            error!("Error happened while reading the incoming email {}", e);
            let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
        }

        let mail = Mail::deserialize(&mut mail);
        match mail {
            Ok(mail)  => {
                let recipients: Vec<String> = mail.recipients;
                let body = mail.body;

                let password = str::from_utf8(passwd);

                if password.is_err() {
                    error!("Cannot read password for account");
                    let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                    return;
                }
                let smtp = Command::new(&self.client)
                  .arg(format!("--passwordeval=echo {}", password.unwrap()))
                  .args(recipients)
                  .stdin(Stdio::piped())
                  .stdout(Stdio::null())
                  .spawn();

                if smtp.is_err() {
                  error!("Failed to start smtp process");
                  let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                }

                let mut smtp = smtp.unwrap();

                match smtp.stdin.as_mut().map(|stdin| stdin.write_all(body.as_slice())) {
                    Some(Ok(_)) => {
                        let _ = stream.write_all(OK_SIGNAL.as_bytes());
                        error!("Email sent to smtp");
                    },
                    Some(Err(why)) => {
                        let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                        error!("Couldn't write to smtp stdin: {}", why.description());
                    },
                    _             => {
                        let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                        error!("Couldn't write to smtp stdin");
                    },
                };
            },
            Err(why)             => {
                let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                error!("Problem sending an email {}", why);
            },
        }
    }

    pub fn start(&self, label: &str, prefix: &str, vault: &Vault, passwd: &[u8]) {
        if let Ok(listener) = UnixListener::bind(get_socket_path(prefix, &label)) {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                      let decrypted = vault.decrypt(passwd);
                      self.send_mail(stream, &decrypted.into_bytes());
                    }
                    _              => {
                        /* connection failed */
                        break;
                    }
                }
            }
        } else {
            error!("failed to open a socket")
        }
    }

    pub fn new(client: &str) -> Self {
        ExternalClient { client: client.to_string() }
    }
}
