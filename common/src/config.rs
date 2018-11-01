use ini::Ini;
use account::Account;
use dirs::home_dir;
use vault::Vault;
use log_and_panic;

pub struct Configuration {
    pub smtpclient: Option<String>,
    pub socket_root: String,
    pub flock_root: String,
    pub spool_root: String,
    pub timeout: u64,
    pub accounts: Vec<Account>,
}

pub fn read_config(rc_path: &str) -> Configuration {
    debug!("Loading configuration from {}", rc_path);
    let conf = Ini::load_from_file(rc_path)
        .unwrap_or_else(|e| log_and_panic(&format!("{}", e)));

    let app = conf.section(Some("App".to_owned()));
    let socket_root = app.and_then(|app| {
        app.get("socket-root-path").map(|s| s.to_string())
    }).unwrap_or_else(|| {
        warn!("No configuration was found for socket-root-path, \
              using the default value");
        "".to_string()
    });

    let flock_root = app.and_then(|app| {
        app.get("flock-root-path").map(|s| s.to_string())
    }).unwrap_or_else(|| {
        warn!("No configuration was found for flock-root-path, \
              using the default value");
        "".to_string()
    });

    let home_dir = home_dir().expect("Cannot find the home directory").display().to_string();
    let defualt_spool = format!("{}/.rusmtp/spool", home_dir);
    let spool_root = app.and_then(|app| {
        app.get("spool-root-path").map(|s| s.to_string())
    }).unwrap_or_else(|| {
        warn!("No configuration was found for spool-root-path, \
              using the default value (i.e. {})", defualt_spool);
        defualt_spool
    });

    let smtp = conf.section(Some("Daemon".to_owned())).and_then(|section| {
        section.get("smtp").map(|s| s.to_string())
    });

    let timeout = conf.section(Some("Client")).and_then(|section| {
        section.get("timeout").map(|s| {
            let res: u64 = s.parse()
                .unwrap_or_else(|_|
                    log_and_panic("Invalid timeout value in configuration"));
            res
        })
    }).unwrap_or_else(|| {
        warn!("No configuration was found for timeout, \
              using the default value, {}", DEFAULT_TIMEOUT_IN_SECONDS);
        DEFAULT_TIMEOUT_IN_SECONDS
    });


    let mut accounts: Vec<Account> = Vec::new();

    for (section_name, section) in conf.iter() {
        if *section_name != Some("App".to_string()) &&
                *section_name != Some("Client".to_string()) &&
                *section_name != Some("Daemon".to_string()) {
            let label    = section_name.clone().unwrap();
            let host     = section.get("host").map(|s| s.to_string());
            let username = section.get("username").map(|s| s.to_string());
            let port     = section.get("port").map(|p| {
                let port: u16 = p.parse()
                    .unwrap_or_else(|_|
                        log_and_panic("Invalid port number value in configuration"));
                port
            });
            let eval     = section.get("passwordeval").map(|s| s.to_string())
                .unwrap_or_else(||
                    log_and_panic("passwordeval is missing in the configuration"));

            let tls      = section.get("tls").map(|p| {
                let tls: bool = p.parse()
                    .unwrap_or_else(|_|
                        log_and_panic(
                            "Invalid tls value in configuration (valid: false | true)"));
                tls
            });

            let default      = section.get("default").map(|p| {
                let default: bool = p.parse()
                    .unwrap_or_else(|_|
                        log_and_panic("Invalid bool value in configuration"));
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
        let _: Configuration = log_and_panic("At least an account should be configured");
    }

    let default_accounts = accounts.iter()
        .fold(0,|z,y| if y.default { z + 1 } else { z} );

    if default_accounts > 1 {
        let _: Configuration = log_and_panic("At most one account can be set to default");
    }


    Configuration {
        smtpclient: smtp,
        socket_root,
        flock_root,
        spool_root,
        timeout,
        accounts,
    }
}

const DEFAULT_TIMEOUT_IN_SECONDS: u64 = 30;
