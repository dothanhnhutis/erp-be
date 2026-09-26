use std::{env, num::ParseIntError};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Prod,
}

impl Environment {
    pub fn from_env() -> Self {
        match std::env::var("APP_ENV").as_deref() {
            Ok("prod" | "production") => Self::Prod,
            _ => Self::Dev,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Prod => "prod",
        }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub app_env: Environment,
    pub crate_name: String,
    pub server_host: String,
    pub server_port: u16,
    pub redis_url: String,
    pub database_url: String,
    pub log_dir: String,
    pub log_filter: String,
    pub log_max_file: usize,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required env var `{0}`")]
    Missing(&'static str),

    #[error("env var `{0}` is not valid UTF-8")]
    NotUnicode(&'static str),

    #[error("invalid value for `{key}`: {value:?}")]
    InvalidNumber {
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
            .map_err(|source| ConfigError::InvalidNumber {
                key: "SERVER_PORT",
                value: port_raw.clone(),
                source,
            })?;
        let log_max_file_raw = optional("LOG_MAX_FILE", "7")?;
        let log_max_file =
            log_max_file_raw
                .parse::<usize>()
                .map_err(|source| ConfigError::InvalidNumber {
                    key: "LOG_MAX_FILE",
                    value: log_max_file_raw.clone(),
                    source,
                })?;

        Ok(Self {
            app_env: Environment::from_env(),
            crate_name: required("CRATE_NAME")?,
            redis_url: required("REDIS_URL")?,
            database_url: required("DATABASE_URL")?,
            log_dir: required("LOG_DIR")?,
            log_filter: required("LOG_FILTER")?,
            log_max_file,
            server_host: optional("SERVER_HOST", "0.0.0.0")?,
            server_port,
        })
    }
}
