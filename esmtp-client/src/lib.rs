pub mod verbs;

extern crate native_tls;
extern crate base64;

use base64::encode;
use std::io::prelude::*;
use native_tls::{TlsConnector, TlsStream};
use std::net::{TcpStream, ToSocketAddrs, IpAddr};

fn to_ip_address(host: &str) -> Vec<IpAddr> {
    (host, 0).to_socket_addrs()
        .map(|iter|
             iter.map(|socket_address| socket_address.ip()).collect())
        .expect(&format!("Cannot resolve host {}", host))
}


fn recieve_print(stream: &mut TlsStream<TcpStream>) {
    let mut responce = [0; 128];
    let _ = stream.read(&mut responce);
    println!("{}", std::str::from_utf8(&responce).unwrap());
}

fn send(stream: &mut TlsStream<TcpStream>, msg: &[u8]) {
    let _ = stream.write(msg);
}

fn send_recieve_print(stream: &mut TlsStream<TcpStream>, msg: &[u8]) {
    send(stream, msg);
    recieve_print(stream);
}

fn main() {
    let ips = to_ip_address("smtp.gmail.com");
    let ip = ips.first().unwrap();

    let connector = TlsConnector::builder().unwrap().build().unwrap();
    let stream = TcpStream::connect(format!("{}:465", ip)).unwrap();
    let mut stream = connector.connect("smtp.gmail.com",
        stream).expect("couldn't connect");

    recieve_print(&mut stream);

    send_recieve_print(&mut stream, b"EHLO smtp-daemon.amanj.me\n");
    send_recieve_print(&mut stream, b"HELP\n");

    let user = "USERNAME";
    let pass = "PASSWORD";

    send_recieve_print(&mut stream, b"AUTH LOGIN ");
    send(&mut stream, &encode(user).as_bytes());
    send_recieve_print(&mut stream, b"\n");
    send(&mut stream, &encode(pass).as_bytes());
    send_recieve_print(&mut stream, b"\n");
}
