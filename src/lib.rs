#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate serde_json;

#[cfg(test)]
mod tests;

use chrono::prelude::*;
use serde_json::Value;
use std::{
    fs::{File},
    io::{Read, Write},
};

use lettre::SmtpClient;
use lettre::Transport;

#[derive(Debug)]
pub struct Config {
    pub imap_control: ServerConfig,
    pub imap: ServerConfig,
    pub smtp: ServerConfig,
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
    starttls,
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

const SUBJECT: &'static str = "Selbstauskunft nach DSGVO";

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    /// run main logic
    pub fn run(mut self) {
        info!(
            "startind datenbriefd version: {}",
            env!("CARGO_PKG_VERSION")
        );
        info!("loaded {} companies", &self.companies.len());
        trace!("load companies time table");

        // parse time file
        match self.parse_time_file() {
            Ok(data) => {
                self.parse_time(&data);
            }
            Err(err) => {
                use std::io::ErrorKind::*;
                match err.kind() {
                    NotFound => info!("could not load {} as timetable: Not Found", self.time_file),
                    _ => info!("could not load {} as timetable: {}", self.time_file, err),
                }
            }
        }

        let mailer = new_mailer(&self.smtp.host, self.smtp.port);
        if mailer.is_none() {
            return; //no mailer available
        }
        let mailer = mailer.unwrap().credentials(lettre::smtp::authentication::Credentials::new(self.smtp.user.clone(), self.smtp.password.clone()));
        let mut mailer = mailer.transport();

        loop {
            let now: DateTime<Utc> = Utc::now();
            for v in self.companies.iter_mut() {
                let v: &mut Company = v;
                if v.next_hit <= now {
                    info!("{} is elapsed, {} reminder", v.name, v.reminder);

                    // update time
                    let now: DateTime<Utc> = now.clone();
                    if let Some(value) =
                        now.checked_add_signed(chrono::Duration::days(v.interval as i64))
                    {
                        v.next_hit = value;
                        trace!("update {} to {}", v.name, value);
                    }

                    let mail = format!(include_str!("mail.fmt"), v.onw_name);

                    if self.dry_run {
                        println!(
                            r#"Mail:
To: {}
From: {} <{}>
Subject: {}

{}
"#,
                            v.mail, v.onw_name, v.alias, SUBJECT, mail
                        );
                    } else {
                        use lettre_email::{Email, mime::TEXT_PLAIN};

                        let message_id = uuid::Uuid::new_v4();

                        let email = Email::builder()
                        // Addresses can be specified by the tuple (email, alias)
                        .to((&v.mail, &v.name))
                        // ... or by an address only
                        .from((v.alias.to_string(), &v.onw_name))
                        .subject(SUBJECT)
                        .text(mail)
                        .message_id(format!("<{}@{}>", message_id, self.smtp.host))
                        .build()
                        .unwrap();

                        trace!("send mail to {} now", v.name);

                        let result = mailer.send(email.into());

                        if result.is_ok() {
                            debug!("send mail to {}", v.name);
                        } else if let Err(err) = result {
                            warn!("could not send mail to {}: {}", v.name, err);
                        }
                    }
                }
            }

            // update time table
            if !self.dry_run {
                self.write_time();
            }

            std::thread::sleep(std::time::Duration::from_secs(100));
        }
    }

    /// parse time table file
    pub(crate) fn parse_time(&mut self, data: &str) {
        let json: serde_json::Result<Value> = serde_json::from_str(data);
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

    fn parse_time_file(&self) -> std::io::Result<String> {
        debug!("load {} as time table", self.time_file);
        let mut file = File::open(&self.time_file)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        Ok(data)
    }

    fn write_time(&self) -> std::io::Result<()> {
        let mut json: Value = Value::default();

        for v in self.companies.iter() {
            let v: &Company = v;
            json[&v.name] = json!({"next": v.next_hit.to_rfc3339(), "reminder": v.reminder});
        }

        debug!(
            "write to time file:\n{}",
            serde_json::to_string_pretty(&json).unwrap()
        );
        let json: String = json.to_string();
        let mut file = File::create(&self.time_file)?;

        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            imap: ServerConfig::new(),
            smtp: ServerConfig::new(),
            imap_control: ServerConfig::new(),
            companies: Vec::new(),
            dry_run: false,
            time_file: String::from("time.json"),
        }
    }
}


fn new_mailer(server: &str, port: u16) -> Option<lettre::smtp::SmtpClient> {
    let mut tls_builder = native_tls::TlsConnector::builder();
    tls_builder.min_protocol_version(Some(lettre::smtp::client::net::DEFAULT_TLS_MIN_PROTOCOL));

    let tls_builder = match tls_builder.build() {
        Err(err) => {
            error!("Unable to seupt tls config: {}", err);
            return None;
        },
        Ok(tls) => tls,
    };


    let tls_parameters =
        lettre::smtp::client::net::ClientTlsParameters::new(server.to_string(), tls_builder);

    match SmtpClient::new(
        (server, port),
        lettre::smtp::ClientSecurity::Wrapper(tls_parameters),
    ) {
        Ok(client) => Some(client),
        Err(err) => {
            error!("unable to init smtp client: {}", err);
            None
        }
    }
}
