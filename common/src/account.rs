use vault::Vault;

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
