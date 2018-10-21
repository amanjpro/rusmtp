extern crate ini;
extern crate docopt;
extern crate dirs;
extern crate ring;
extern crate rand;

use ring::aead::*;
use ring::pbkdf2::*;
use ring::digest::SHA256;
use ring::rand::{SystemRandom, SecureRandom};


#[macro_use]
extern crate serde_derive;

use rand::{thread_rng, Rng};
use std::str;
use ini::Ini;
use docopt::Docopt;
use dirs::home_dir;
use std::process::exit;

pub struct Vault {
    pub salt: Vec<u8>,
    pub opening_key: OpeningKey,
    pub sealing_key: SealingKey,
    pub nonce: Vec<u8>,
}

impl Vault {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        let password_size: usize = rng.gen_range(8, 100);
        let mut password = vec![0u8; password_size];
        let ring_rand = SystemRandom::new();
        ring_rand.fill(&mut password).expect("Cannot fill random password");

        let salt_size: usize = rng.gen_range(8, 100);
        let mut salt = vec![0u8; salt_size];
        let ring_rand = SystemRandom::new();
        ring_rand.fill(&mut salt).expect("Cannot fill the salt");

        let mut key = [0; 32];
        derive(&SHA256, 100, &salt, &password[..], &mut key);

        let opening_key = OpeningKey::new(&CHACHA20_POLY1305, &key).expect("Cannot generate opening key");
        let sealing_key = SealingKey::new(&CHACHA20_POLY1305, &key).expect("Cannot generate sealing key");

        let mut nonce = vec![0; 12];
        let ring_rand = SystemRandom::new();
        ring_rand.fill(&mut nonce).expect("Cannot generate nonce");

        Vault {
            salt: salt,
            opening_key: opening_key,
            sealing_key: sealing_key,
            nonce: nonce
        }

    }

    pub fn encrypt(&self, passwd: &mut String) -> Vec<u8> {
        let passwd: &mut [u8] = unsafe {passwd.as_bytes_mut() };
        let mut passwd = &mut passwd.to_vec();
        let additional_data: [u8; 0] = [];
        for _ in 0..CHACHA20_POLY1305.tag_len() {
            passwd.push(0);
        }
        let _ = seal_in_place(&self.sealing_key, &self.nonce, &additional_data, &mut passwd,
                                    CHACHA20_POLY1305.tag_len()).expect("Cannot encrypt password");
        passwd.clone()
    }

    pub fn decrypt(&self, passwd: Vec<u8>) -> String {
        let mut passwd = passwd.clone();
        let additional_data: [u8; 0] = [];
        let res = open_in_place(&self.opening_key, &self.nonce, &additional_data, 0,
                         &mut passwd).expect("Cannot decrypt password");
        String::from_utf8(res.to_vec())
            .expect("Cannot convert the decrypted password to text")
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
    pub password: Option<Vec<u8>>,
    pub vault: Vault,
}

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
                vault: Vault::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption() {
        let vault = Vault::new();
        let mut original: &mut String = &mut String::from("very$secure*passw0rd#");
        let mut encrypted: &mut String = &mut original.clone();
        let mut encrypted = &mut vault.encrypt(&mut encrypted);
        let mut decrypted = &mut vault.decrypt(encrypted.clone());
        assert_ne!(original.clone().into_bytes(), *encrypted);
        assert_eq!(*original, *decrypted);
    }
}
