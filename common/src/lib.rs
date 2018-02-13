#[macro_use]
extern crate serde_derive;

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
    let smtp = section.get("smtp").unwrap();

    let section = conf.section(Some("SMTP".to_owned())).unwrap();
    let username = section.get("username").unwrap();
    let host = section.get("host").parse().unwrap());
    let port = section.get("port").unwrap();

    Configuration {
        passwordeval: eval.to_string(),
        smtpclient: smtp.to_string(),
    }
}

pub static SOCKET_PATH: &'static str = "smtp-daemon-socket";
