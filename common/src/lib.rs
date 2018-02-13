#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}


pub struct Configuration {
    pub passwordeval: String,
    pub smtpclient: String,
}

pub fn read_config(rc_path: &str) -> Configuration {
    let conf = Ini::load_from_file(rc_path).unwrap();

    let section = conf.section(Some("Daemon".to_owned())).unwrap();
    let eval = section.get("passwordeval").unwrap();
    let smtp = section.get("smtp").unwrap();

    Configuration {
        passwordeval: eval.to_string(),
        smtpclient: smtp.to_string(),
    }
}

pub static SOCKET_PATH: &'static str = "smtp-daemon-socket";
