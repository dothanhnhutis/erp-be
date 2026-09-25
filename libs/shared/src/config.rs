use std::{env, num::ParseIntError};
use thiserror::Error;

#[derive(Debug)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required env var `{0}`")]
    Missing(&'static str),

    #[error("env var `{0}` is not valid UTF-8")]
    NotUnicode(&'static str),

    #[error("invalid value for `{key}`: {value:?}")]
    InvalidPort {
        key: &'static str,
        value: String,
        #[source]
        source: ParseIntError,
    },
}

fn required(key: &'static str) -> Result<String, ConfigError> {
    env::var(key).map_err(|e| match e {
        env::VarError::NotPresent => ConfigError::Missing(key),
        env::VarError::NotUnicode(_) => ConfigError::NotUnicode(key),
    })
}

fn optional(key: &'static str, default: &str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(v) => Ok(v),
        Err(env::VarError::NotPresent) => Ok(default.to_owned()),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::NotUnicode(key)),
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port_raw = optional("SERVER_PORT", "4000")?;
        let server_port = port_raw
            .parse::<u16>()
            .map_err(|source| ConfigError::InvalidPort {
                key: "SERVER_PORT",
                value: port_raw.clone(),
                source,
            })?;

        Ok(Self {
            // redis_url: optional("REDIS_URL", "redis://127.0.0.1:6379")?,
            // database_url: required("DATABASE_URL")?,
            server_host: optional("SERVER_HOST", "0.0.0.0")?,
            server_port,
        })
    }
}
