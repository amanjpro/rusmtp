extern crate ring;
extern crate rand;
extern crate dirs;
extern crate docopt;
extern crate ini;
extern crate protocol;
extern crate serde_json;

pub mod account;
pub mod vault;
pub mod config;
pub mod args;
pub mod clients;

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub account: Option<String>,
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}

pub fn get_socket_path(account: &str) -> String {
  format!("{}-{}", SOCKET_PATH_PREFIX, account)
}

static SOCKET_PATH_PREFIX: &'static str = "rusmtp-daemon-socket";
pub static OK_SIGNAL: &'static str = "OK";
pub static ERROR_SIGNAL: &'static str = "ERROR";
