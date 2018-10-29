pub mod verbs;

extern crate common;
extern crate native_tls;
extern crate base64;

#[macro_use]
extern crate log;

use verbs::*;
use common::log_and_panic;
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

    pub fn open_connection(host: &str, port: u16) -> SMTPConnection {
        debug!("Openning connection with {}", host);
        let ips = SMTPConnection::get_ip_address(host);
        let ip = ips.first()
            .unwrap_or_else(||
                log_and_panic(&format!("Could not resolve the host: {}", host)));

        debug!("Securing connection with {}", host);
        let connector = TlsConnector::builder()
            .build()
            .unwrap_or_else(|_| log_and_panic("Cannot establish connection"));

        debug!("Securing connection with {} on port {}", ip, port);
        let stream = TcpStream::connect(format!("{}:{}", ip, port))
            .unwrap_or_else(|_|
                log_and_panic(&format!("Cannot connect to {} on port {}", host, port)));

        debug!("Establishing TLS connection with {}", host);
        let mut stream = connector.connect(host, stream)
            .unwrap_or_else(|_|
                log_and_panic(&format!("Cannot establish TLS connection to {}", host)));

        let response = SMTPConnection::recieve(&mut stream);

        debug!("{}", &response);

        debug!("Checking the presence of ESMTP protocol");
        SMTPConnection::true_or_panic(
            response.starts_with("220") && response.contains("ESMTP"),
            &format!("SMTP Server {} is not accepting clients", host));

        debug!("Shaking hands with the ESMTP server");
        let response = SMTPConnection::send_and_check(&mut stream,
            &format!("{} rusmtp.amanj.me\n", EHLO).as_bytes(),
            &|response| response.starts_with("250"),
            &format!("SMTP Server {} does not support ESMTP", host));

        if response.contains(STARTTLS) {
            debug!("Checking if TLS is supported");
            SMTPConnection::send_and_check(&mut stream,
                &format!("{} rusmtp.amanj.me\n", STARTTLS).as_bytes(),
                &|response| response.starts_with("250"),
                "Cannot start a TLS connection");

            debug!("Shaking hands with the ESMTP server again, but this time over TLS");
            SMTPConnection::send_and_check(&mut stream,
                &format!("{} rusmtp.amanj.me\n", EHLO).as_bytes(),
                &|response| response.starts_with("250"),
                &format!("SMTP Server {} does not support ESMTP", host));
        }


        SMTPConnection {
            stream,
            supports_login: response.contains(LOGIN),
            supports_xoauth2: response.contains(XOAUTH2),
        }
    }

    pub fn login(&mut self, username: &[u8], passwd: &[u8]) {
       SMTPConnection::send(&mut self.stream, format!("{} {}\n", AUTH, LOGIN).as_bytes());
       let response = SMTPConnection::recieve(&mut self.stream);
       debug!("{}", &response);
       SMTPConnection::send(&mut self.stream, &encode(&username).as_bytes());
       SMTPConnection::send(&mut self.stream, b"\n");
       let response = SMTPConnection::recieve(&mut self.stream);
       debug!("{}", &response);
       SMTPConnection::send(&mut self.stream, &encode(&passwd).as_bytes());
       SMTPConnection::send_and_check(&mut self.stream, b"\n",
           &|response| response.starts_with("235"),
           "Invalid username or password");
    }

    pub fn send_mail(&mut self, from: &str, recipients: &[&str], body: &[u8]) {
       SMTPConnection::send_and_check(&mut self.stream,
          format!("{} {}:<{}>\r\n", MAIL, FROM, from).as_bytes(),
           &|response| response.starts_with("250"),
           &format!("Cannot send email from {}", from));

       for recipient in recipients.iter() {
          SMTPConnection::send_and_check(&mut self.stream,
              format!("{} {}:<{}>\r\n", RCPT, TO, recipient).as_bytes(),
              &|response| response.starts_with("250"),
              &format!("Cannot send email to {}", recipient));
       }

       SMTPConnection::send_and_check(&mut self.stream, format!("{}\r\n", DATA).as_bytes(),
              &|response| response.starts_with("354"),
              "Cannot start sending email");

       SMTPConnection::send(&mut self.stream, body);
       SMTPConnection::send_and_check(&mut self.stream, b"\r\n.\r\n",
           &|response| response.starts_with("250"),
           "Failed to send email");
    }

    fn send_and_check(mut stream: &mut TlsStream<TcpStream>, msg: &[u8],
                      check: &Fn(&str) -> bool,
                      on_failure_msg: &str) -> String {
       SMTPConnection::send(&mut stream, msg);
       let response = SMTPConnection::recieve(&mut stream);
       debug!("{}", &response);
       SMTPConnection::true_or_panic(
           check(&response),
           on_failure_msg);
       response
    }

    fn true_or_panic(flag: bool, panic_message: &str) {
        if ! flag {
            log_and_panic(panic_message)
        }
    }


    fn get_ip_address(host: &str) -> Vec<IpAddr> {
        (host, 0).to_socket_addrs()
            .map(|iter|
                 iter.map(|socket_address| socket_address.ip()).collect())
            .unwrap_or_else(|_|
                log_and_panic(&format!("Cannot resolve host {}", host)))
    }


    fn recieve(stream: &mut TlsStream<TcpStream>) -> String {
        let mut response = [0; 4096];
        let _ = stream.read(&mut response);
        std::str::from_utf8(&response).
            unwrap_or_else(|_| log_and_panic("Cannot decode the message")).to_string()
    }

    fn send(stream: &mut TlsStream<TcpStream>, msg: &[u8]) {
        let _ = stream.write(msg);
    }

    pub fn shutdown(&mut self) {
        let _ = self.stream.shutdown();
    }
}
