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
    pub flag_account: Option<String>,
    pub flag_smtpdrc: String,
    flag_help: bool,
    flag_version: bool,
}


#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub account: Option<String>,
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}

const DEFAULT_HEARTBEAT_IN_MINUTES: u8 = 3;
const DEFAULT_TIMEOUT_IN_SECONDS: u64 = 30;

#[derive(Debug)]
pub struct Account {
    pub label: String,
    pub username: Option<String>,
    pub passwordeval: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub tls: Option<bool>,
    pub heartbeat: u8,
    pub default: bool,
}

#[derive(Debug)]
pub struct Configuration {
    pub smtpclient: Option<String>,
    pub timeout: u64,
    pub accounts: Vec<Account>,
}

pub fn read_config(rc_path: &str) -> Configuration {
    let conf = Ini::load_from_file(rc_path).unwrap();

    let smtp = conf.section(Some("Daemon".to_owned())).and_then(|section| {
        section.get("smtp").map(|s| s.to_string())
    });

    let timeout = conf.section(Some("Client")).and_then(|section| {
        section.get("timeout").map(|s| {
            let res: u64 = s.parse()
                .expect("Invalid timeout value in configuration");
            res
        })
    }).unwrap_or(DEFAULT_TIMEOUT_IN_SECONDS);


    let mut accounts: Vec<Account> = Vec::new();

    for (section_name, section) in conf.iter() {
        if *section_name != Some("Client".to_string()) &&
                *section_name != Some("Daemon".to_string()) {
            let label = section_name.clone().unwrap();
            let host     = section.get("host").map(|s| s.to_string());
            let username = section.get("username").map(|s| s.to_string());
            let port     = section.get("port").map(|p| {
                let port: u16 = p.parse()
                    .expect("Invalid port number value in configuration");
                port
            });
            let eval     = section.get("passwordeval").map(|s| s.to_string())
                .expect("passwordeval is missing in the configuration");

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

            let default      = section.get("default").map(|p| {
                let default: bool = p.parse()
                    .expect("Invalid bool value in configuration");
                default
            }).unwrap_or(false);



            accounts.push(Account {
                label: label,
                host: host,
                username: username,
                passwordeval: eval.to_string(),
                port: port,
                tls: tls,
                heartbeat: heartbeat,
                default: default,
            })
        }
    }

    if accounts.is_empty() {
        panic!("At least an account should be configured");
    }

    Configuration {
        smtpclient: smtp,
        timeout: timeout,
        accounts: accounts,
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

        Usage: {0} [--smtpdrc=<string>] [--account=<string>] [--] <recipients>...
               {0} --help
               {0} --version

        Options:
            --account=<string>       The account on which the email should be sent.
                                     If none is provided, the default account would
                                     be chosen.
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


pub fn get_socket_path(account: &str) -> String {
  format!("{}-{}", SOCKET_PATH_PREFIX, account)
}

static SOCKET_PATH_PREFIX: &'static str = "smtp-daemon-socket";
pub static OK_SIGNAL: &'static str = "OK";
pub static ERROR_SIGNAL: &'static str = "ERROR";
