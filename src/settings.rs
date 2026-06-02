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
pub struct History {
    pub max_length: usize,
    pub max_ttl: i64,
    #[serde(default = "default_ttl")]
    pub default_ttl: u64,
}

fn default_ttl() -> u64 {
    30
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct LongPoll {
    pub timeout_secs: u64,
}

impl Default for LongPoll {
    fn default() -> Self {
        Self { timeout_secs: 30 }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub security: Security,
    pub log: Log,
    pub history: History,
    pub general: General,
    #[serde(default = "LongPoll::default")]
    pub long_poll: LongPoll,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_from_default_file() {
        let settings = Settings::new().expect("Should parse push_config.toml");
        assert_eq!(settings.general.port, 9099);
        assert!(!settings.security.enabled);
        assert_eq!(settings.history.max_length, 5000);
        assert_eq!(settings.history.max_ttl, 600);
        assert_eq!(settings.long_poll.timeout_secs, 30);
    }

    #[test]
    fn test_long_poll_default() {
        let lp = LongPoll::default();
        assert_eq!(lp.timeout_secs, 30);
    }
}
