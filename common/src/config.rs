use ini::Ini;
use account::Account;
use vault::Vault;

pub struct Configuration {
    pub smtpclient: Option<String>,
    pub socket_root: String,
    pub flock_root: String,
    pub timeout: u64,
    pub accounts: Vec<Account>,
}

pub fn read_config(rc_path: &str) -> Configuration {
    let conf = Ini::load_from_file(rc_path).unwrap();

    let app = conf.section(Some("App".to_owned()));
    let socket_root = app.and_then(|app| {
        app.get("socket-root-path").map(|s| s.to_string())
    }).unwrap_or_else(|| "".to_string());

    let flock_root = app.and_then(|app| {
        app.get("flock-root-path").map(|s| s.to_string())
    }).unwrap_or_else(|| "".to_string());

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
        if *section_name != Some("App".to_string()) &&
                *section_name != Some("Client".to_string()) &&
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

            let default      = section.get("default").map(|p| {
                let default: bool = p.parse()
                    .expect("Invalid bool value in configuration");
                default
            }).unwrap_or(false);

            accounts.push(Account {
                label,
                host,
                username,
                passwordeval: eval.to_string(),
                port,
                tls,
                default,
                password: None,
                vault: Vault::new(),
            })
        }
    }

    if accounts.is_empty() {
        panic!("At least an account should be configured");
    }

    let default_accounts = accounts.iter()
        .fold(0,|z,y| if y.default { z + 1 } else { z} );

    if default_accounts > 1 {
        panic!("At most one account can be set to default");
    }


    Configuration {
        smtpclient: smtp,
        flock_root,
        socket_root,
        timeout,
        accounts,
    }
}

const DEFAULT_TIMEOUT_IN_SECONDS: u64 = 30;
