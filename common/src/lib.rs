extern crate ini;
extern crate rand;
extern crate secstr;
extern crate docopt;
extern crate dirs;
extern crate crypto;

#[macro_use]
extern crate serde_derive;

use secstr::SecStr;
use rand::{thread_rng, Rng};
use std::collections::LinkedList;
use std::str;
use ini::Ini;
use docopt::Docopt;
use dirs::home_dir;
use std::process::exit;

#[derive(Clone)]
pub struct Vault {
    pub algorithms: LinkedList<(String, u64)>,
}

impl Vault {
    pub fn new() -> Self {
        let all_algorithms = vec![
            "AES",
            "Bcrypt",
            "Blake2B",
            "Blowfish",
            "ChaCha20",
            "Curve25519",
            "Ed25519",
            "Fortuna",
            "Ghash",
            "HC128",
            "HMAC",
            "Poly1305",
            "RC4",
            "RIPEMD-160",
            "Scrypt",
            "Sosemanuk",
            "Whirlpool",
        ];

        let mut rng = thread_rng();
        let num_algorithms: usize = rng.gen_range(1, all_algorithms.len());
        let algorithms = vec![0; num_algorithms];
        let algorithms: LinkedList<(String, u64)> = algorithms.iter().map(|_| {
            let index: usize = rng.gen_range(0, all_algorithms.len() - 1);
            let key:u64 = rng.gen();
            (all_algorithms[index].to_string(), key)
        }).collect();

        Vault {
            algorithms: algorithms,
        }
    }

    pub fn encrypt(&self, passwd: &SecStr) -> SecStr {
        let passwd = str::from_utf8(passwd.unsecure()).unwrap();

        let encrypted = self.algorithms.iter().fold(passwd, |acc, next| {
            let (algo, key) = next;
            match algo.as_ref() {
                "AES"        => "",
                "Bcrypt"     => "",
                "Blake2B"    => "",
                "Blowfish"   => "",
                "ChaCha20"   => "",
                "Curve25519" => "",
                "Ed25519"    => "",
                "Fortuna"    => "",
                "Ghash"      => "",
                "HC128"      => "",
                "HMAC"       => "",
                "Poly1305"   => "",
                "RC4"        => "",
                "RIPEMD-160" => "",
                "Scrypt"     => "",
                "Sosemanuk"  => "",
                "Whirlpool"  => "",
                _            => "",
            }
        });

        SecStr::from(encrypted)
    }

    pub fn decrypt(&self, encrypted: &SecStr) -> SecStr {
        let encrypted = str::from_utf8(encrypted.unsecure()).unwrap();

        let passwd = self.algorithms.iter().rev().fold(encrypted, |acc, next| {
            let (algo, key) = next;
            match algo.as_ref() {
                "AES"        => "",
                "Bcrypt"     => "",
                "Blake2B"    => "",
                "Blowfish"   => "",
                "ChaCha20"   => "",
                "Curve25519" => "",
                "Ed25519"    => "",
                "Fortuna"    => "",
                "Ghash"      => "",
                "HC128"      => "",
                "HMAC"       => "",
                "Poly1305"   => "",
                "RC4"        => "",
                "RIPEMD-160" => "",
                "Scrypt"     => "",
                "Sosemanuk"  => "",
                "Whirlpool"  => "",
                _            => "",
            }
        });

        SecStr::from(passwd)
    }
}

// Define the struct that results from those options.
#[derive(Deserialize, Debug)]
pub struct Args {
    pub arg_recipients: Vec<String>,
    pub flag_account: Option<String>,
    pub flag_rusmtprc: String,
    flag_help: bool,
    flag_version: bool,
}


#[derive(Deserialize, Serialize)]
pub struct Mail {
    pub account: Option<String>,
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}

const DEFAULT_HEARTBEAT_IN_MINUTES: u64 = 3;
const DEFAULT_TIMEOUT_IN_SECONDS: u64 = 30;

#[derive(Debug, PartialEq)]
pub enum AccountMode {
    Paranoid,
    Secure,
}

#[derive(Debug)]
pub struct Account {
    pub label: String,
    pub username: Option<String>,
    pub passwordeval: String,
    pub mode: AccountMode,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub tls: Option<bool>,
    pub heartbeat: u64,
    pub default: bool,
    pub password: Option<SecStr>,
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
                let heartbeat: u64 = p.parse()
                    .expect("Invalid u64 value in configuration");
                heartbeat
            }).unwrap_or(DEFAULT_HEARTBEAT_IN_MINUTES);

            let mode         = section.get("mode").map(|p| {
               match p.as_ref() {
                   "paranoid" => AccountMode::Paranoid,
                   ""         => AccountMode::Paranoid,
                   "secure"    => AccountMode::Secure,
                   &_         => panic!("Possible options for mode is `paranoid` and `secure`"),
               }
            }).unwrap_or(AccountMode::Paranoid);

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
                mode: mode,
                default: default,
                password: None,
            })
        }
    }

    if accounts.is_empty() {
        panic!("At least an account should be configured");
    }

    let default_accounts = accounts.iter()
        .filter(|acc| acc.default)
        .collect::<Vec<_>>()
        .len();

    if default_accounts > 1 {
        panic!("At most one account can be set to default");
    }


    Configuration {
        smtpclient: smtp,
        timeout: timeout,
        accounts: accounts,
    }
}

pub fn rusmtpd_usage(app_name: &str) -> String {
    let home_dir = home_dir().expect("Cannot find the home directory");
    let home_dir = home_dir.display();
    format!("
        {}

        Usage: {0}
               {0} --rusmtprc=<string>
               {0} --help
               {0} --version

        Options:
            --rusmtprc=<string>      Path to the rusmtprc [default: {}/.rusmtprc]
            -h, --help               Show this help.
            -v, --version            Show the version.
        ", app_name, home_dir)
}

pub fn rusmtpc_usage(app_name: &str) -> String {
    let home_dir = home_dir().expect("Cannot find the home directory");
    let home_dir = home_dir.display();
    format!("
        {}

        Usage: {0} [--rusmtprc=<string>] [--account=<string>] [--] <recipients>...
               {0} --help
               {0} --version

        Options:
            --account=<string>       The account on which the email should be sent.
                                     If none is provided, the default account would
                                     be chosen.
            --rusmtprc=<string>      Path to the rusmtprc [default: {}/.rusmtprc]
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

static SOCKET_PATH_PREFIX: &'static str = "rusmtp-daemon-socket";
pub static OK_SIGNAL: &'static str = "OK";
pub static ERROR_SIGNAL: &'static str = "ERROR";
