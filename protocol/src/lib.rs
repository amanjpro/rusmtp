pub mod verbs;

extern crate native_tls;
extern crate base64;

#[macro_use]
extern crate log;

use verbs::*;
use base64::encode;
use std::io::prelude::*;
use std::net::Shutdown;
use native_tls::{TlsConnector, TlsStream};
use std::net::{TcpStream, ToSocketAddrs, IpAddr};


pub trait Stream: Read + Write + Sized {
    fn close(&mut self);
}

#[derive(PartialEq)]
pub enum Authentication {
    None,
    Login,
    XAuth2,
}

impl Stream for TlsStream<TcpStream> {
    fn close(&mut self) {
        let _ = self.shutdown();
    }
}

impl Stream for TcpStream {
    fn close(&mut self) {
        let _ = self.shutdown(Shutdown::Both);
    }
}

pub trait Raven: Stream {

    fn create_connection(host: &str, port: u16) -> Result<Self, String>;

    fn hand_shake(&mut self, host: &str) -> Result<Vec<Authentication>, String> {
        let response = self.recieve()?;
        debug!("{}", &response);

        debug!("Checking the presence of ESMTP protocol");
        true_or_err(
            response.starts_with("220") && response.contains("ESMTP"),
            &format!("SMTP Server {} is not accepting clients", host))?;

        debug!("Shaking hands with the ESMTP server");
        let response = self.send_or_err(
            &format!("{} rusmtp.amanj.me\n", EHLO).as_bytes(),
            &|response| response.starts_with("250"),
            &format!("SMTP Server {} does not support ESMTP", host))?;

        if response.contains(STARTTLS) {
            debug!("Checking if TLS is supported");
            let _ = self.send_or_err(
                &format!("{} rusmtp.amanj.me\n", STARTTLS).as_bytes(),
                &|response| response.starts_with("250"),
                "Cannot start a TLS connection")?;

            debug!("Shaking hands with the ESMTP server again, but this time over TLS");
            let _ = self.send_or_err(
                &format!("{} rusmtp.amanj.me\n", EHLO).as_bytes(),
                &|response| response.starts_with("250"),
                &format!("SMTP Server {} does not support ESMTP", host))?;
        }

        let mut auths: Vec<Authentication> = Vec::new();

        if response.contains(LOGIN) {
            auths.push(Authentication::Login);
        }

        if response.contains(XOAUTH2) {
            auths.push(Authentication::XAuth2);
        }

        if response.is_empty() {
            auths.push(Authentication::None)
        }

        Ok(auths)
    }

    fn authenticate_with_login(&mut self, username: &[u8], passwd: &[u8]) -> Result<String, String> {
       self.send(format!("{} {}\n", AUTH, LOGIN).as_bytes());
       let response = self.recieve()?;
       debug!("{}", &response);
       self.send(&encode(&username).as_bytes());
       self.send(b"\n");
       let response = self.recieve()?;
       debug!("{}", &response);
       self.send(&encode(&passwd).as_bytes());
       self.send_or_err(b"\n",
           &|response| response.starts_with("235"),
           "Invalid username or password")
    }

    fn send_mail(&mut self, from: &str, recipients: &[&str], body: &[u8]) -> Result<String, String> {
       let _ = self.send_or_err(
          format!("{} {}:<{}>\r\n", MAIL, FROM, from).as_bytes(),
           &|response| response.starts_with("250"),
           &format!("Cannot send email from {}", from))?;

       for recipient in recipients.iter() {
          let _ = self.send_or_err(
              format!("{} {}:<{}>\r\n", RCPT, TO, recipient).as_bytes(),
              &|response| response.starts_with("250"),
              &format!("Cannot send email to {}", recipient))?;
       }

       let _ = self.send_or_err(format!("{}\r\n", DATA).as_bytes(),
              &|response| response.starts_with("354"),
              "Cannot start sending email")?;

       self.send(body);
       self.send_or_err(b"\r\n.\r\n",
           &|response| response.starts_with("250"),
           "Failed to send email")
    }

    fn send_or_err(&mut self, msg: &[u8],
                      check: &Fn(&str) -> bool,
                      on_failure_msg: &str) -> Result<String, String> {
       self.send(msg);
       let response = self.recieve()?;
       debug!("{}", &response);
       let res =  true_or_err(
           check(&response),
           on_failure_msg);
       res.map(|_| response.to_string())
    }

    fn recieve(&mut self) -> Result<String, String> {
        let mut response = [0; 4096];
        let _ = self.read(&mut response);
        let res = std::str::from_utf8(&response);
        if res.is_err() {
            Err("Cannot decode the message".to_string())
        } else {
            Ok(res.unwrap().to_string())
        }
    }

    fn send(&mut self, msg: &[u8]) {
        let _ = self.write(msg);
    }
}

impl Raven for TlsStream<TcpStream> {
    fn create_connection(host: &str, port: u16) -> Result<Self, String> {
        debug!("Securing connection with {}", host);
        let connector = TlsConnector::builder().build();

        debug!("Securing connection with {} on port {}", host, port);
        let stream = TcpStream::create_connection(host, port)?;

        if connector.is_ok() {
            debug!("Establishing TLS connection with {}", host);
            let res = connector.unwrap().connect(host, stream);
            if res.is_ok() {
                Ok(res.unwrap())
            } else {
                Err(format!("Establishing TLS connection with {} failed", host))
            }
        } else {
            Err(format!("Establishing TLS connection with {} failed", host))
        }
    }
}

impl Raven for TcpStream {
    fn create_connection(host: &str, port: u16) -> Result<Self, String> {
        debug!("Openning connection with {}", host);
        let ips = get_ip_address(host)?;
        let ip = ips.first();
        if ip.is_none() {
            return Err(format!("Cannot resolve the host {}", host));
        }

        let ip = ip.unwrap();

        if let Ok(stream) = TcpStream::connect(format!("{}:{}", ip, port)) {
            Ok(stream)
        } else {
            Err(format!("Cannot establish TCP connection with {}", host))
        }
    }
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
