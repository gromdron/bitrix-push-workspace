use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct General {
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Security {
    pub enabled: bool,
    pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub security: Security,
    pub log: Log,
    pub general: General,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| "./push_config.toml".into());

        let s = Config::builder()
            .add_source(File::with_name(&config_file).required(true))
            .add_source(Environment::with_prefix("push"))
            .build()?;

        s.try_deserialize()
    }
}
