pub mod verbs;

extern crate native_tls;
extern crate secstr;
extern crate base64;

use verbs::*;
use secstr::SecStr;
use base64::{encode, decode};
use std::io::prelude::*;
use native_tls::{TlsConnector, TlsStream};
use std::net::{TcpStream, ToSocketAddrs, IpAddr};


pub struct SMTPConnection {
    stream: TlsStream<TcpStream>,
    supports_login: bool,
    supports_xoauth2: bool,
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

        let responce = SMTPConnection::recieve(&mut stream);

        SMTPConnection::log(&responce);
        SMTPConnection::true_or_panic(
            responce.starts_with("220") && responce.contains("ESMTP"),
            &format!("SMTP Server {} is not accepting clients", host));

        SMTPConnection::send(&mut stream, EHLO.as_bytes());
        SMTPConnection::send(&mut stream, b" smtp.amanj.me\n");
        let responce = SMTPConnection::recieve(&mut stream);
        SMTPConnection::log(&responce);
        SMTPConnection::true_or_panic(
            responce.starts_with("250"),
            &format!("SMTP Server {} does not support ESMTP", host));


        SMTPConnection {
            stream: stream,
            supports_login: responce.contains(LOGIN),
            supports_xoauth2: responce.contains(XOAUTH2),
        }
    }

    pub fn login(&mut self, username: SecStr, passwd: SecStr) {
       SMTPConnection::send(&mut self.stream, format!("{} {}\n", AUTH, LOGIN).as_bytes());
       let responce = SMTPConnection::recieve(&mut self.stream);
       SMTPConnection::log(&responce);
       SMTPConnection::send(&mut self.stream, &encode(username.unsecure()).as_bytes());
       SMTPConnection::send(&mut self.stream, b"\n");
       let responce = SMTPConnection::recieve(&mut self.stream);
       SMTPConnection::log(&responce);
       SMTPConnection::send(&mut self.stream, &encode(passwd.unsecure()).as_bytes());
       SMTPConnection::send(&mut self.stream, b"\n");
       let responce = SMTPConnection::recieve(&mut self.stream);
       SMTPConnection::log(&responce);
       SMTPConnection::true_or_panic(
           responce.starts_with("235"),
           "Invalid username or password");
    }


    pub fn send_mail(&mut self, from: &str, recipients: &[&str], body: &str) {
       SMTPConnection::send(&mut self.stream, format!("{} {}:<{}>\r\n", MAIL, FROM, from).as_bytes());
       let responce = SMTPConnection::recieve(&mut self.stream);
       SMTPConnection::log(&responce);
       SMTPConnection::true_or_panic(
           responce.starts_with("250"),
           &format!("Cannot send email from {}", from));
       for recipient in recipients.iter() {
          SMTPConnection::send(&mut self.stream, format!("{} {}:<{}>\r\n", RCPT, TO, recipient).as_bytes());
          SMTPConnection::log(&responce);
          SMTPConnection::true_or_panic(
             responce.starts_with("250"),
             &format!("Cannot send email to {}", recipient));

       }

       SMTPConnection::send(&mut self.stream, format!("{}\r\n", DATA).as_bytes());
       SMTPConnection::send(&mut self.stream, body.as_bytes());
       SMTPConnection::send(&mut self.stream, b"\r\n.\r\n");
       SMTPConnection::log(&responce);
       SMTPConnection::true_or_panic(
           responce.starts_with("250"),
           &format!("Cannot send email to {:?}", recipients));
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
        let mut responce = [0; 4096];
        let _ = stream.read(&mut responce);
        std::str::from_utf8(&responce).expect("Cannot decode the message").to_string()
    }

    fn send(stream: &mut TlsStream<TcpStream>, msg: &[u8]) {
        let _ = stream.write(msg);
    }

    fn extract_msg(reply: &[u8]) -> String {
        let mut iter = std::str::from_utf8(&reply).expect("Cannot decode the message")
            .splitn(2, " ");
        let _ = iter.next();

        iter.next().expect("Cannot get the message").to_string()
    }
}

fn main() {
}
