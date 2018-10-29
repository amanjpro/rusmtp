extern crate ring;
extern crate rand;
extern crate dirs;
extern crate docopt;
extern crate ini;


use std::u64;

pub mod account;
pub mod vault;
pub mod config;
pub mod args;
pub mod mail;

#[macro_use]
extern crate serde_derive;

pub fn get_lock_path(prefix: &str, account: &str) -> String {
  if prefix.is_empty() {
      format!("{}-{}", FLOCK_PATH_PREFIX, account)
  } else {
      format!("{}/{}-{}", prefix, FLOCK_PATH_PREFIX, account)
  }
}

pub fn get_socket_path(prefix: &str, account: &str) -> String {
  if prefix.is_empty() {
      format!("{}-{}", SOCKET_PATH_PREFIX, account)
  } else {
      format!("{}/{}-{}", prefix, SOCKET_PATH_PREFIX, account)
  }
}

static FLOCK_PATH_PREFIX: &'static str = "rusmtp-daemon-flock";
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
    [b1, b2, b3, b4, b5, b6, b7, b8]
}

fn transform_array_of_u8_to_u64(bytes: &[u8]) -> u64 {
    let mut number = u64::from(bytes[7]);
    number |= u64::from(bytes[6]) << 8;
    number |= u64::from(bytes[5]) << 16;
    number |= u64::from(bytes[4]) << 24;
    number |= u64::from(bytes[3]) << 32;
    number |= u64::from(bytes[2]) << 40;
    number |= u64::from(bytes[1]) << 48;
    number |= u64::from(bytes[0]) << 56;
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
        assert_eq!(38_328_183, check(38_328_183));
        assert_eq!(u64::MAX - 1, check(u64::MAX -1 ));
        assert_eq!(u64::MIN + 1, check(u64::MIN + 1));
    }
}
