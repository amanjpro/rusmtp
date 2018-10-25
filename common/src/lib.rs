extern crate ring;
extern crate rand;
extern crate dirs;
extern crate docopt;
extern crate ini;
extern crate protocol;

use std::u64;

pub mod account;
pub mod vault;
pub mod config;
pub mod args;
pub mod clients;
pub mod mail;

#[macro_use]
extern crate serde_derive;

pub fn get_socket_path(account: &str) -> String {
  format!("{}-{}", SOCKET_PATH_PREFIX, account)
}

static SOCKET_PATH_PREFIX: &'static str = "rusmtp-daemon-socket";
pub static OK_SIGNAL: &'static str = "OK";
pub static ERROR_SIGNAL: &'static str = "ERROR";

fn transform_u64_to_array_of_u8(x: u64) -> [u8; 8] {
    let b1 : u8 = ((x >> 56) & 0xff) as u8;
    let b2 : u8 = ((x >> 48) & 0xff) as u8;
    let b3 : u8 = ((x >> 40) & 0xff) as u8;
    let b4 : u8 = ((x >> 32) & 0xff) as u8;
    let b5 : u8 = ((x >> 24) & 0xff) as u8;
    let b6 : u8 = ((x >> 16) & 0xff) as u8;
    let b7 : u8 = ((x >> 8) & 0xff) as u8;
    let b8 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4, b5, b6, b7, b8]
}

fn transform_array_of_u8_to_u64(bytes: &[u8]) -> u64 {
    let mut number = bytes[7] as u64;
    number |= (bytes[6] as u64) << 8;
    number |= (bytes[5] as u64) << 16;
    number |= (bytes[4] as u64) << 24;
    number |= (bytes[3] as u64) << 32;
    number |= (bytes[2] as u64) << 40;
    number |= (bytes[1] as u64) << 48;
    number |= (bytes[0] as u64) << 56;
    number
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializing_deserializing_u64() {
        let check = |n| {
            transform_array_of_u8_to_u64(
                &transform_u64_to_array_of_u8(n))
        };
        assert_eq!(0, check(0));
        assert_eq!(u64::MAX, check(u64::MAX));
        assert_eq!(u64::MIN, check(u64::MIN));
        assert_eq!(183, check(183));
        assert_eq!(38328183, check(38328183));
        assert_eq!(u64::MAX - 1, check(u64::MAX -1 ));
        assert_eq!(u64::MIN + 1, check(u64::MIN + 1));
    }
}
