#[macro_use] extern crate log;
extern crate env_logger;


pub struct Config {
    pub ImapControl: ServerConfig,
    pub Imap: ServerConfig,
    pub Smtp: ServerConfig,
}

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
            encryption: Encryption::tls,
            user: String::new(),
            password: String::new(),
        }
    }
}

pub enum Encryption {
    tls,
    starttls
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    /// run main logic
    pub fn run(self) {
        warn!("create a run function");
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