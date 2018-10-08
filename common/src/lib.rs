extern crate ini;
extern crate docopt;
extern crate dirs;

#[macro_use]
extern crate serde_derive;

use ini::Ini;
use docopt::Docopt;
use dirs::home_dir;
use std::process::exit;

// Define the struct that results from those options.
#[derive(Deserialize, Debug)]
pub struct Args {
    pub arg_recipients: Vec<String>,
    pub flag_smtpdrc: String,
    flag_help: bool,
    flag_version: bool,
}


#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}

const DEFAULT_HEARTBEAT_IN_MINUTES: u8 = 3;
const DEFAULT_TIMEOUT_IN_SECONDS: u64 = 30;

#[derive(Debug)]
pub struct Configuration {
    pub passwordeval: String,
    pub smtpclient: Option<String>,
    pub username: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub tls: Option<bool>,
    pub heartbeat: u8,
    pub timeout: u64,
}

pub fn read_config(rc_path: &str) -> Configuration {
    let conf = Ini::load_from_file(rc_path).unwrap();

    let section = conf.section(Some("Daemon".to_owned()))
        .expect("SMTP Daemon section is missing in the configuration");

    let eval = section.get("passwordeval")
        .expect("passwordeval is missing in the configuration");
    let smtp = section.get("smtp").map(|s| s.to_string());

    let timeout = conf.section(Some("Client")).map(|section| {
        section.get("timeout").map(|s| {
            let res: u64 = s.parse()
                .expect("Invalid timeout value in configuration");
            res
        }).unwrap_or(DEFAULT_TIMEOUT_IN_SECONDS)
    }).unwrap_or(DEFAULT_TIMEOUT_IN_SECONDS);

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
                    timeout: timeout,
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
                timeout: timeout,
            },
    }
}

pub fn smtpd_usage(app_name: &str) -> String {
    let home_dir = home_dir().expect("Cannot find the home directory");
    let home_dir = home_dir.display();
    format!("
        {}

        Usage: {0}
               {0} --smtpdrc=<string>
               {0} --help
               {0} --version

        Options:
            --smtpdrc=<string>       Path to the smtpdrc [default: {}/.smtpdrc]
            -h, --help               Show this help.
            -v, --version            Show the version.
        ", app_name, home_dir)
}

pub fn smtpc_usage(app_name: &str) -> String {
    let home_dir = home_dir().expect("Cannot find the home directory");
    let home_dir = home_dir.display();
    format!("
        {}

        Usage: {0} [--smtpdrc=<string>] [--] <recipients>...
               {0} --help
               {0} --version

        Options:
            --smtpdrc=<string>       Path to the smtpdrc [default: {}/.smtpdrc]
            -h, --help               Show this help.
            -v, --version            Show the version.
        ", app_name, home_dir)
}

pub fn process_args(app_name: &str, usage: &str) -> Args {

    let app_version = env!("CARGO_PKG_VERSION");

    let args: Args = Docopt::new(usage.clone())
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}, v {}", app_name, app_version);
        exit(0);
    }

    if args.flag_help {
        println!("{}", usage);
        exit(0);
    }

    args
}

pub static SOCKET_PATH: &'static str = "smtp-daemon-socket";
pub static OK_SIGNAL: &'static str = "OK";
pub static ERROR_SIGNAL: &'static str = "ERROR";
