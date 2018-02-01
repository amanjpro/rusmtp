#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}

pub static SOCKET_PATH: &'static str = "smtp-daemon-socket";
