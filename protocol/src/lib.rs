pub mod verbs;

extern crate native_tls;
extern crate base64;

#[macro_use]
extern crate log;

use verbs::*;
use base64::encode;
use std::io::prelude::*;
use native_tls::{TlsConnector, TlsStream};
use std::net::{TcpStream, ToSocketAddrs, IpAddr};


pub struct SMTPConnection {
    stream: TlsStream<TcpStream>,
    pub supports_login: bool,
    pub supports_xoauth2: bool,
}

impl SMTPConnection {

    fn create_connection(host: &str, port: u16) -> Result<TlsStream<TcpStream>, String> {
        debug!("Openning connection with {}", host);
        let ips = SMTPConnection::get_ip_address(host)?;
        let ip = ips.first();
        if ip.is_none() {
            return Err(format!("Cannot resolve the host {}", host));
        }

        let ip = ip.unwrap();

        debug!("Securing connection with {}", host);
        let connector = TlsConnector::builder().build();

        debug!("Securing connection with {} on port {}", ip, port);
        let stream = TcpStream::connect(format!("{}:{}", ip, port));

        if connector.is_ok() && stream.is_ok() {
            debug!("Establishing TLS connection with {}", host);
            let res = connector.unwrap().connect(host, stream.unwrap());
            if res.is_ok() {
                Ok(res.unwrap())
            } else {
                Err(format!("Establishing TLS connection with {} failed", host))
            }
        } else {
            Err(format!("Establishing TLS connection with {} failed", host))
        }
    }

    pub fn open_connection(host: &str, port: u16) -> Result<SMTPConnection, String> {
        let stream = SMTPConnection::create_connection(host, port);
        if stream.is_err() {
            return Err(stream.unwrap_err());
        }
        let mut stream = stream.unwrap();

        let response = SMTPConnection::recieve(&mut stream)?;
        debug!("{}", &response);

        debug!("Checking the presence of ESMTP protocol");
        SMTPConnection::true_or_err(
            response.starts_with("220") && response.contains("ESMTP"),
            &format!("SMTP Server {} is not accepting clients", host))?;

        debug!("Shaking hands with the ESMTP server");
        let response = SMTPConnection::send_or_err(&mut stream,
            &format!("{} rusmtp.amanj.me\n", EHLO).as_bytes(),
            &|response| response.starts_with("250"),
            &format!("SMTP Server {} does not support ESMTP", host))?;

        if response.contains(STARTTLS) {
            debug!("Checking if TLS is supported");
            let _ = SMTPConnection::send_or_err(&mut stream,
                &format!("{} rusmtp.amanj.me\n", STARTTLS).as_bytes(),
                &|response| response.starts_with("250"),
                "Cannot start a TLS connection")?;

            debug!("Shaking hands with the ESMTP server again, but this time over TLS");
            let _ = SMTPConnection::send_or_err(&mut stream,
                &format!("{} rusmtp.amanj.me\n", EHLO).as_bytes(),
                &|response| response.starts_with("250"),
                &format!("SMTP Server {} does not support ESMTP", host))?;
        }


        Ok(SMTPConnection {
            stream,
            supports_login: response.contains(LOGIN),
            supports_xoauth2: response.contains(XOAUTH2),
        })
    }

    pub fn login(&mut self, username: &[u8], passwd: &[u8]) -> Result<String, String> {
       SMTPConnection::send(&mut self.stream, format!("{} {}\n", AUTH, LOGIN).as_bytes());
       let response = SMTPConnection::recieve(&mut self.stream)?;
       debug!("{}", &response);
       SMTPConnection::send(&mut self.stream, &encode(&username).as_bytes());
       SMTPConnection::send(&mut self.stream, b"\n");
       let response = SMTPConnection::recieve(&mut self.stream)?;
       debug!("{}", &response);
       SMTPConnection::send(&mut self.stream, &encode(&passwd).as_bytes());
       SMTPConnection::send_or_err(&mut self.stream, b"\n",
           &|response| response.starts_with("235"),
           "Invalid username or password")
    }

    pub fn send_mail(&mut self, from: &str, recipients: &[&str], body: &[u8]) -> Result<String, String> {
       let _ = SMTPConnection::send_or_err(&mut self.stream,
          format!("{} {}:<{}>\r\n", MAIL, FROM, from).as_bytes(),
           &|response| response.starts_with("250"),
           &format!("Cannot send email from {}", from))?;

       for recipient in recipients.iter() {
          let _ = SMTPConnection::send_or_err(&mut self.stream,
              format!("{} {}:<{}>\r\n", RCPT, TO, recipient).as_bytes(),
              &|response| response.starts_with("250"),
              &format!("Cannot send email to {}", recipient))?;
       }

       let _ = SMTPConnection::send_or_err(&mut self.stream, format!("{}\r\n", DATA).as_bytes(),
              &|response| response.starts_with("354"),
              "Cannot start sending email")?;

       SMTPConnection::send(&mut self.stream, body);
       SMTPConnection::send_or_err(&mut self.stream, b"\r\n.\r\n",
           &|response| response.starts_with("250"),
           "Failed to send email")
    }

    fn send_or_err(mut stream: &mut TlsStream<TcpStream>, msg: &[u8],
                      check: &Fn(&str) -> bool,
                      on_failure_msg: &str) -> Result<String, String> {
       SMTPConnection::send(&mut stream, msg);
       let response = SMTPConnection::recieve(&mut stream)?;
       debug!("{}", &response);
       let res =  SMTPConnection::true_or_err(
           check(&response),
           on_failure_msg);
       res.map(|_| response.to_string())
    }

    fn true_or_err(flag: bool, error_message: &str) -> Result<(), String> {
        if ! flag {
            Err(error_message.to_string())
        } else {
            Ok(())
        }
    }

    fn get_ip_address(host: &str) -> Result<Vec<IpAddr>, String> {
        let res = (host, 0).to_socket_addrs()
            .map(|iter|
                 iter.map(|socket_address| socket_address.ip()).collect());
        if res.is_err() {
            Err(format!("Cannot resolve host {}", host))
        } else {
            Ok(res.unwrap())
        }
    }


    fn recieve(stream: &mut TlsStream<TcpStream>) -> Result<String, String> {
        let mut response = [0; 4096];
        let _ = stream.read(&mut response);
        let res = std::str::from_utf8(&response);
        if res.is_err() {
            Err("Cannot decode the message".to_string())
        } else {
            Ok(res.unwrap().to_string())
        }
    }

    fn send(stream: &mut TlsStream<TcpStream>, msg: &[u8]) {
        let _ = stream.write(msg);
    }

    pub fn shutdown(&mut self) {
        let _ = self.stream.shutdown();
    }
}
