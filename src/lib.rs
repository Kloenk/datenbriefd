#[macro_use] extern crate log;
extern crate env_logger;

#[derive(Debug)]
pub struct Config {
    pub ImapControl: ServerConfig,
    pub Imap: ServerConfig,
    pub Smtp: ServerConfig,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub encryption: Encryption,
    pub user: String,
    pub password: String,
}

impl ServerConfig {
    pub fn new() -> Self {
        Self {
            port: 0,
            host: String::new(),
            encryption: Encryption::starttls,
            user: String::new(),
            password: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct Company {
    pub name: String,
    pub mail: String,
    pub alias: String,
    pub onw_name: String,
    pub interval: usize,
}

#[derive(Debug)]
pub enum Encryption {
    tls,
    starttls
}

impl Encryption {
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "tls" => Encryption::tls,
            "starttls" => Encryption::starttls,
            _ => Encryption::starttls,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    /// run main logic
    pub fn run(self) {
        warn!("create a run function to run:\n{:?}", self);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            Imap: ServerConfig::new(),
            Smtp: ServerConfig::new(),
            ImapControl: ServerConfig::new(),
        }
    }
}