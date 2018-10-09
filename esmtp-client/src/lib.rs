pub mod verbs;

extern crate native_tls;
extern crate secstr;
extern crate base64;

use verbs::*;
use secstr::SecStr;
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
        let ips = SMTPConnection::to_ip_address(host);
        let ip = ips.first()
            .expect(&format!("Could not resolve the host: {}", host));

        let connector = TlsConnector::builder()
            .expect("Cannot establish connection")
            .build().expect("Cannot establish connection");

        let stream = TcpStream::connect(format!("{}:{}", ip, port))
            .expect(&format!("Cannot connect to {} on port {}", host, port));

        let mut stream = connector.connect(host, stream)
            .expect(&format!("Cannot establish TLS connection to {}", host));

        let response = SMTPConnection::recieve(&mut stream);

        SMTPConnection::log(&response);

        SMTPConnection::true_or_panic(
            response.starts_with("220") && response.contains("ESMTP"),
            &format!("SMTP Server {} is not accepting clients", host));

        let response = SMTPConnection::send_and_check(&mut stream,
            &format!("{} smtp.amanj.me\n", EHLO).as_bytes(),
            &|response| response.starts_with("250"),
            &format!("SMTP Server {} does not support ESMTP", host));

        if response.contains(STARTTLS) {
            SMTPConnection::send_and_check(&mut stream,
                &format!("{} smtp.amanj.me\n", STARTTLS).as_bytes(),
                &|response| response.starts_with("250"),
                &format!("Cannot start a TLS connection"));

            SMTPConnection::send_and_check(&mut stream,
                &format!("{} smtp.amanj.me\n", EHLO).as_bytes(),
                &|response| response.starts_with("250"),
                &format!("SMTP Server {} does not support ESMTP", host));
        }


        SMTPConnection {
            stream: stream,
            supports_login: response.contains(LOGIN),
            supports_xoauth2: response.contains(XOAUTH2),
        }
    }

    pub fn login(&mut self, username: &SecStr, passwd: &SecStr) {
       SMTPConnection::send(&mut self.stream, format!("{} {}\n", AUTH, LOGIN).as_bytes());
       let response = SMTPConnection::recieve(&mut self.stream);
       SMTPConnection::log(&response);
       SMTPConnection::send(&mut self.stream, &encode(&username.unsecure()).as_bytes());
       SMTPConnection::send(&mut self.stream, b"\n");
       let response = SMTPConnection::recieve(&mut self.stream);
       SMTPConnection::log(&response);
       SMTPConnection::send(&mut self.stream, &encode(&passwd.unsecure()).as_bytes());
       SMTPConnection::send_and_check(&mut self.stream, b"\n",
           &|response| response.starts_with("235"),
           "Invalid username or password");
    }

    pub fn keep_alive(&mut self) {
        SMTPConnection::send_and_check(&mut self.stream,
          format!("{} {}:<{}>\r\n", MAIL, FROM, "keep-alive").as_bytes(),
           &|response| response.starts_with("250"),
           "Connection with the server is lost");

        SMTPConnection::send_and_check(&mut self.stream, &format!("{}\n", RSET).as_bytes(),
           &|response| response.starts_with("250"),
           "Connection with the server is lost");

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
              &format!("Cannot start sending email"));

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
       SMTPConnection::log(&response);
       SMTPConnection::true_or_panic(
           check(&response),
           on_failure_msg);
       response
    }

    fn log(msg: &str) {
        println!("{}", msg)
    }

    fn true_or_panic(flag: bool, panic_message: &str) {
        if ! flag {
            panic!(panic_message.to_string())
        }
    }


    fn to_ip_address(host: &str) -> Vec<IpAddr> {
        (host, 0).to_socket_addrs()
            .map(|iter|
                 iter.map(|socket_address| socket_address.ip()).collect())
            .expect(&format!("Cannot resolve host {}", host))
    }


    fn recieve(stream: &mut TlsStream<TcpStream>) -> String {
        let mut response = [0; 4096];
        let _ = stream.read(&mut response);
        std::str::from_utf8(&response).expect("Cannot decode the message").to_string()
    }

    fn send(stream: &mut TlsStream<TcpStream>, msg: &[u8]) {
        let _ = stream.write(msg);
    }
}
