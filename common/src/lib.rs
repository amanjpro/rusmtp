extern crate ini;

#[macro_use]
extern crate serde_derive;

use ini::Ini;

#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}


pub struct Configuration {
    pub passwordeval: String,
    pub smtpclient: Option<String>,
    pub username: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub tls: Option<bool>,
}

pub fn read_config(rc_path: &str) -> Configuration {
    let conf = Ini::load_from_file(rc_path).unwrap();

    let section = conf.section(Some("Daemon".to_owned())).unwrap();
    let eval = section.get("passwordeval").unwrap();
    let smtp = section.get("smtp").map(|s| s.to_string());

    let section  = conf.section(Some("SMTP".to_owned())).unwrap();
    let username = section.get("username").map(|s| s.to_string());
    let host     = section.get("host").map(|s| s.to_string());
    let port     = section.get("port").map(|p| {
        let port: u16 = p.parse().unwrap();
        port
    });

    let tls      = section.get("tls").map(|p| {
        let port: bool = p.parse().unwrap();
        port
    });

    Configuration {
        passwordeval: eval.to_string(),
        smtpclient: smtp,
        username: username,
        host: host,
        port: port,
        tls: tls,
    }
}

pub static SOCKET_PATH: &'static str = "smtp-daemon-socket";
