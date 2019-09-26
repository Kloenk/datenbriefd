#[macro_use] extern crate log;
extern crate env_logger;

use clap::{App, Arg, SubCommand};
use datenbriefd::{Config};

fn main() {
    env_logger::init();
    let mut app = App::new("datenbriefd")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Kloenk <me@kloenk.de>")
        .about("mail daemon to periodicly send a datenbrief")
        .setting(clap::AppSettings::ColorAuto)
        .setting(clap::AppSettings::ColoredHelp)
         .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("set config file")
                .takes_value(true)
                .default_value("config.toml")
        ) 
        .arg(
            Arg::with_name("control.server")
                .long("control-server")
                .value_name("SERVER")
                .help("server for the imap control")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("control.port")
                .long("control-port")
                .value_name("PORT")
                .help("port for the imap control")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("control.encryption")
                .long("control-encryption")
                .value_name("SCHEMA")
                .help("encryption type for the control imap")
                .takes_value(true)
                .possible_value("tls")
                .possible_value("starttls")
        )
        .arg(
            Arg::with_name("control.user")
                .long("control-user")
                .value_name("USER")
                .help("user name for control imap")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("control.password")
                .long("control-password")
                .value_name("PASSWORD")
                .help("password for control imap")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("imap.server")
                .long("imap-server")
                .value_name("SERVER")
                .help("server for the imap")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("imap.port")
                .long("imap-port")
                .value_name("PORT")
                .help("port for the imap")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("imap.encryption")
                .long("imap-encryption")
                .value_name("SCHEMA")
                .help("encryption type for the imap")
                .takes_value(true)
                .possible_value("tls")
                .possible_value("starttls")
        )
        .arg(
            Arg::with_name("imap.user")
                .long("imap-user")
                .value_name("USER")
                .help("user name for imap")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("imap.password")
                .long("imap-password")
                .value_name("PASSWORD")
                .help("password for imap")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("smtp.server")
                .long("smtp-server")
                .value_name("SERVER")
                .help("server for the smtp")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("smtp.port")
                .long("smtp-port")
                .value_name("PORT")
                .help("port for the smtp")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("smtp.encryption")
                .long("smtp-encryption")
                .value_name("SCHEMA")
                .help("encryption type for the smtp")
                .takes_value(true)
                .possible_value("tls")
                .possible_value("starttls")
        )
        .arg(
            Arg::with_name("smtp.user")
                .long("smtp-user")
                .value_name("USER")
                .help("user name for smtp")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("smtp.password")
                .long("smtp-password")
                .value_name("PASSWORD")
                .help("password for smtp")
                .takes_value(true)
        );
    
    if cfg!(feature = "completion") {
        app = app.subcommand(
            SubCommand::with_name("completion")
                .about("create completions")
                .version("0.1.0")
                .author("Kloenk <me@kloenk.de>")
                .arg(
                    Arg::with_name("shell")
                        .help("set the shell to create for. Tries to identify with env variable")
                        .index(1)
                        .required(false)
                        .value_name("SHELL")
                        .possible_value("fish")
                        .possible_value("bash")
                        .possible_value("zsh")
                        .possible_value("powershell")
                        .possible_value("elvish"),
                )
                .arg(
                    Arg::with_name("out")
                        .help("sets output file")
                        .value_name("FILE")
                        .short("o")
                        .long("output"),
                )
                .setting(clap::AppSettings::ColorAuto)
                .setting(clap::AppSettings::ColoredHelp)
        );
    }

    let matches = app.clone().get_matches();

    if cfg!(feature = "completion") {
        if let Some(matches) = matches.subcommand_matches("completion") {
            completion(&matches, &mut app);
            std::process::exit(0);
        }
    }
    drop(app);  // remove arguemnt parser

    // Gets a value for config if supplied by user, or defaults to "config.toml"
    let config = matches.value_of("config").unwrap_or("config.toml");
    println!("Value for config: {}", config);

    let toml_config: Option<toml::Value> = match std::fs::read_to_string(config) {
        Ok(config) => match toml::from_str(config.as_str()) {
            Ok(config) => Some(config),
            Err(err) => {
                warn!("Error parsing config file: {}", err);
                None
            }
        },
        Err(err) => {
            info!("Error reading file: {}", err);
            None
        }
    };

    let mut config = Config::new();

    if let Some(value) = &matches.value_of("control.server") {
        debug!("set imap control server to {}", value);
        config.ImapControl.host = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("control") {
            if let Some(value) = value.get("server") {
                if let Some(value) = value.as_str() {
                    debug!("set imap control server to {}", value);
                    config.ImapControl.host = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("control.port") {
        let value: Result<u16, std::num::ParseIntError> = value.parse();
        if let Ok(value) = value {
            debug!("set imap control port to {}", value);
            config.ImapControl.port = value;
        } else if let Err(err) = value {
            warn!("imap control port is not a u16 number: {}", err);
        }
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("control") {
            if let Some(value) = value.get("port") {
                if let Some(value) = value.as_integer() {
                    debug!("set imap control port to {}", value);
                    config.ImapControl.port = value as u16;
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("control.encryption") {
        let value = datenbriefd::Encryption::parse(value);
        debug!("set imap control encryption to {}", "FIXME");
        config.ImapControl.encryption = value;
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("control") {
            if let Some(value) = value.get("encryption") {
                if let Some(value) = value.as_str() {
                    let value = datenbriefd::Encryption::parse(value);
                    debug!("set imap control encryption to {}", "FIXME");
                    config.ImapControl.encryption = value;
                }
            }
        }
    }



    if let Some(value) = &matches.value_of("control.user") {
        debug!("set imap control user to {}", value);
        config.ImapControl.user = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("control") {
            if let Some(value) = value.get("user") {
                if let Some(value) = value.as_str() {
                    debug!("set imap control user to {}", value);
                    config.ImapControl.user = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("control.password") {
        //debug!("set imap control password to {}", value);
        config.ImapControl.password = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("control") {
            if let Some(value) = value.get("password") {
                if let Some(value) = value.as_str() {
                    //debug!("set imap control password to {}", value);
                    config.ImapControl.password = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("imap.server") {
        debug!("set imap server to {}", value);
        config.Imap.host = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("imap") {
            if let Some(value) = value.get("server") {
                if let Some(value) = value.as_str() {
                    debug!("set imap server to {}", value);
                    config.Imap.host = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("imap.port") {
        let value: Result<u16, std::num::ParseIntError> = value.parse();
        if let Ok(value) = value {
            debug!("set imap port to {}", value);
            config.Imap.port = value;
        } else if let Err(err) = value {
            warn!("imap port is not a u16 number: {}", err);
        }
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("imap") {
            if let Some(value) = value.get("port") {
                if let Some(value) = value.as_integer() {
                    debug!("set imap port to {}", value);
                    config.Imap.port = value as u16;
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("imap.encryption") {
        let value = datenbriefd::Encryption::parse(value);
        debug!("set imap encryption to {}", "FIXME");
        config.Imap.encryption = value;
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("imap") {
            if let Some(value) = value.get("encryption") {
                if let Some(value) = value.as_str() {
                    let value = datenbriefd::Encryption::parse(value);
                    debug!("set imap encryption to {}", "FIXME");
                    config.Imap.encryption = value;
                }
            }
        }
    }



    if let Some(value) = &matches.value_of("imap.user") {
        debug!("set imap user to {}", value);
        config.Imap.user = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("imap") {
            if let Some(value) = value.get("user") {
                if let Some(value) = value.as_str() {
                    debug!("set imap user to {}", value);
                    config.Imap.user = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("control.password") {
        //debug!("set imap password to {}", value);
        config.Imap.password = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("imap") {
            if let Some(value) = value.get("password") {
                if let Some(value) = value.as_str() {
                    //debug!("set imap password to {}", value);
                    config.Imap.password = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("smtp.server") {
        debug!("set smtp server to {}", value);
        config.Smtp.host = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("smtp") {
            if let Some(value) = value.get("server") {
                if let Some(value) = value.as_str() {
                    debug!("set smtp server to {}", value);
                    config.Smtp.host = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("stmp.port") {
        let value: Result<u16, std::num::ParseIntError> = value.parse();
        if let Ok(value) = value {
            debug!("set smtp port to {}", value);
            config.Smtp.port = value;
        } else if let Err(err) = value {
            warn!("imap smtp is not a u16 number: {}", err);
        }
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("smtp") {
            if let Some(value) = value.get("port") {
                if let Some(value) = value.as_integer() {
                    debug!("set smtp port to {}", value);
                    config.Smtp.port = value as u16;
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("smtp.encryption") {
        let value = datenbriefd::Encryption::parse(value);
        debug!("set imap control encryption to {}", "FIXME");
        config.Smtp.encryption = value;
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("smtp") {
            if let Some(value) = value.get("encryption") {
                if let Some(value) = value.as_str() {
                    let value = datenbriefd::Encryption::parse(value);
                    debug!("set smtp encryption to {}", "FIXME");
                    config.Smtp.encryption = value;
                }
            }
        }
    }



    if let Some(value) = &matches.value_of("smtp.user") {
        debug!("set smtp user to {}", value);
        config.Smtp.user = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("smtp") {
            if let Some(value) = value.get("user") {
                if let Some(value) = value.as_str() {
                    debug!("set smtp user to {}", value);
                    config.Smtp.user = value.to_string();
                }
            }
        }
    }

    if let Some(value) = &matches.value_of("smtp.password") {
        //debug!("set smtp password to {}", value);
        config.Smtp.password = value.to_string();
    } else if let Some(toml_config) = &toml_config {
        if let Some(value) = toml_config.get("smtp") {
            if let Some(value) = value.get("password") {
                if let Some(value) = value.as_str() {
                    //debug!("set smtp password to {}", value);
                    config.Smtp.password = value.to_string();
                }
            }
        }
    }

    drop(matches);  // removed parsed arguments
    config.run();
}


/// create completion
#[cfg(feature = "completion")]
fn completion(args: &clap::ArgMatches, app: &mut App) {
    let shell: String = match args.value_of("shell") {
        Some(shell) => shell.to_string(),
        None => shell()
    };

    use clap::Shell;
    let shell_l = shell.to_lowercase();
    let shell: Shell;
    if shell_l == "fish".to_string() {
        shell = Shell::Fish;
    } else if shell_l == "zsh".to_string() {
        shell = Shell::Zsh;
    } else if shell_l == "powershell".to_string() {
        shell = Shell::PowerShell;
    } else if shell_l == "elvish".to_string() {
        shell = Shell::Elvish;
    } else {
        shell = Shell::Bash;
    }

    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;

    let mut path = BufWriter::new(match args.value_of("out") {
        Some(x) => Box::new(
            File::create(&std::path::Path::new(x)).unwrap_or_else(|err| {
                eprintln!("Error opening file: {}", err);
                std::process::exit(1);
            }),
        ) as Box<dyn Write>,
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    });


    app.gen_completions_to("datenbriefd", shell, &mut path);
}

#[cfg(all(feature = "completion", not(windows)))]
fn shell() -> String {
    let shell: String = match std::env::var("SHELL") {
            Ok(shell) => shell,
            Err(_) => "/bin/bash".to_string(),
    };
    let shell = std::path::Path::new(&shell);
    match shell.file_name() {
        Some(shell) => shell.to_os_string().to_string_lossy().to_string(),
        None => "bash".to_string(),
    }
}

#[cfg(all(feature = "completion", windows))]
fn shell() -> String {
    "powershell".to_string()    // always default to powershell on windows
}

#[cfg(not(feature = "completion"))]
fn completion(_: &clap::ArgMatches, _: &mut App) {
    eprintln!("Completion command fired but completion not included in features");
    std::process::exit(-1);
}
