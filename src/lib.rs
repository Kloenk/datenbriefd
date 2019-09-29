#[macro_use] extern crate log;
extern crate env_logger;

#[macro_use] extern crate serde_json;

use serde_json::{Value};
use std::{io::{Read, Write}, fs::{File, OpenOptions}};
use chrono::prelude::*;

#[derive(Debug)]
pub struct Config {
    pub ImapControl: ServerConfig,
    pub Imap: ServerConfig,
    pub Smtp: ServerConfig,
    pub companies: Vec<Company>,
    pub dry_run: bool,
    pub time_file: String,
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

#[derive(Debug, Clone)]
pub struct Company {
    pub name: String,
    pub mail: String,
    pub alias: String,
    pub onw_name: String,
    pub interval: usize,
    reminder: u8,
    next_hit: DateTime<Utc>,
}

impl Company {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            mail: String::new(),
            alias: String::new(),
            onw_name: String::new(),
            interval: 365,
            reminder: 0,
            next_hit: Utc::now(),
        }
    }
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
    pub fn run(mut self) {
        //self.write_time().unwrap();
        info!("startind datenbriefd version: {}", env!("CARGO_PKG_VERSION"));
        info!("loaded {} companies", &self.companies.len());
        trace!("load companies time table");
        self.parse_time_file();


        error!("create a run function to run:\n{:?}", self);
    }

    /// parse time table file
    fn parse_time_file(&mut self) {
        debug!("load {} as time table", self.time_file);

        let file = File::open(&self.time_file);
        if let Err(err) = file {
            if err.kind() == std::io::ErrorKind::NotFound {
                trace!("could not load {}: File Not Found", self.time_file);
            } else {
                warn!("could not load {}: {}", self.time_file, err);
            }
            return;
        }
        let mut file = file.unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let json: serde_json::Result<Value> = serde_json::from_str(&data);
        if let Err(err) = json {
            warn!("error parsing json time table: {}", err);
            return;
        }
        let json = json.unwrap();
        for v in self.companies.iter_mut() {
            let v: &mut Company = v;
            if let Some(value) = json.get(&v.name) {
                if let Some(value) = value.get("next") {
                    if let Some(value) = value.as_str() {
                        let value = value.parse::<DateTime<Utc>>();
                        if let Ok(value) = value {
                            trace!("read next hit for {} on {}", v.name, value);
                            v.next_hit = value;
                        } else if let Err(err) = value {
                            error!("could not load next hit for {}: {}", v.name, err);
                        }
                    }
                }
                if let Some(value) = value.get("reminder") {
                    if let Some(value) = value.as_i64() {
                        trace!("read reminder for {} as {}", v.name, value);
                        v.reminder = value as u8;
                    }
                }
            } else {
                debug!("{} has no entry in the time table file", v.name);
            }
        }
    }

    fn write_time(&self) -> std::io::Result<()> {
        let mut json: Value = Value::default();

        for v in self.companies.iter() {
            let v: &Company = v;
            json[&v.name] = json!({"next": v.next_hit.to_rfc3339(), "reminder": v.reminder});
        }

       
        debug!("write to time file:\n{}", serde_json::to_string_pretty(&json).unwrap());
         let json: String = json.to_string();
        let mut file = File::create(&self.time_file)?;

        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            Imap: ServerConfig::new(),
            Smtp: ServerConfig::new(),
            ImapControl: ServerConfig::new(),
            companies: Vec::new(),
            dry_run: false,
            time_file: String::from("time.json"),
        }
    }
}