extern crate ini;

#[macro_use]
extern crate serde_derive;

use ini::Ini;

#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}

const DEFAULT_HEARTBEAT_IN_MINUTES: u8 = 3;

#[derive(Clone)]
pub struct Configuration {
    pub passwordeval: String,
    pub smtpclient: Option<String>,
    pub username: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub tls: Option<bool>,
    pub heartbeat: u8,
}

pub fn read_config(rc_path: &str) -> Configuration {
    let conf = Ini::load_from_file(rc_path).unwrap();

    let section = conf.section(Some("Daemon".to_owned()))
        .expect("SMTP Daemon section is missing in the configuration");

    let eval = section.get("passwordeval")
        .expect("passwordeval is missing in the configuration");
    let smtp = section.get("smtp").map(|s| s.to_string());

    match conf.section(Some("SMTP".to_owned())) {
        Some(section)     => {
                let username = section.get("username").map(|s| s.to_string());
                let host     = section.get("host").map(|s| s.to_string());
                let port     = section.get("port").map(|p| {
                    let port: u16 = p.parse()
                        .expect("Invalid port number value in configuration");
                    port
                });

                let tls      = section.get("tls").map(|p| {
                    let tls: bool = p.parse()
                        .expect("Invalid tls value in configuration (valid: false | true)");
                    tls
                });

                let heartbeat      = section.get("heartbeat").map(|p| {
                    let heartbeat: u8 = p.parse()
                        .expect("Invalid u8 value in configuration");
                    heartbeat
                }).unwrap_or(DEFAULT_HEARTBEAT_IN_MINUTES);


                Configuration {
                    passwordeval: eval.to_string(),
                    smtpclient: smtp,
                    username: username,
                    host: host,
                    port: port,
                    tls: tls,
                    heartbeat: heartbeat,
                }
            },
        None             =>
            Configuration {
                passwordeval: eval.to_string(),
                smtpclient: smtp,
                username: None,
                host: None,
                port: None,
                tls: None,
                heartbeat: DEFAULT_HEARTBEAT_IN_MINUTES,
            },
    }
}

pub static SOCKET_PATH: &'static str = "smtp-daemon-socket";
pub static OK_SIGNAL: &'static str = "OK";
pub static ERROR_SIGNAL: &'static str = "ERROR";
