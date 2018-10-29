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
        let mut mail = String::new();
        let _ = stream.read_to_string(&mut mail).unwrap();
        // TODO: Failure here? should be reported back to rusmtpc
        let mail = Mail::deserialize(&mut mail.into_bytes()).unwrap();
        let recipients: Vec<String> = mail.recipients;
        let body = mail.body;

        let smtp = Command::new(&self.client)
          .arg(format!("--passwordeval=echo {}", str::from_utf8(passwd).unwrap()))
          .args(recipients)
          .stdin(Stdio::piped())
          .stdout(Stdio::null())
          .spawn()
          .expect("Failed to start smtp process");

        match smtp.stdin.unwrap().write_all(body.as_slice()) {
            Err(why) => {
                let _ = stream.write_all(ERROR_SIGNAL.as_bytes());
                panic!("couldn't write to smtp stdin: {}", why.description());
            },
            Ok(_) => {
                let _ = stream.write_all(OK_SIGNAL.as_bytes());
                println!("email sent to smtp");
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
            panic!("failed to open a socket")
        }
    }

    pub fn new(client: &str) -> Self {
        ExternalClient { client: client.to_string() }
    }
}
