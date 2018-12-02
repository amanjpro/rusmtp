extern crate native_tls;

use common::*;
use common::mail::*;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::net::Shutdown;
use std::time::Duration;

pub mod default;
pub mod external;

pub fn send_to_daemon(mail: &Mail, socket_root: &str, timeout: u64, account: &str) ->
        Result<(), String> {
    let socket_path = get_socket_path(&socket_root, account);
    let stream = UnixStream::connect(socket_path);
    if stream.is_err() {
        return Err(stream.unwrap_err().to_string());
    }
    let mut stream = stream.unwrap();
    let res = stream.write_all(mail.serialize().as_slice());
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    };

    let _ = stream.shutdown(Shutdown::Write);
    let timeout = Duration::new(timeout, 0);
    let _ = stream.set_read_timeout(Some(timeout));
    let mut response = Vec::new();
    let res = stream.read_to_end(&mut response);
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    };
    let response = String::from_utf8(response);
    if response.is_err() {
        return Err(res.unwrap_err().to_string());
    };
    let response = response.unwrap();
    if OK_SIGNAL == response {
        Ok(())
    } else if ERROR_SIGNAL == response {
        Err("Something is not right in the server".to_string())
    } else {
        Err(format!("Unexpected response from the server: {}", response))
    }
}
