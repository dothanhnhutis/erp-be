use std::{env, num::ParseIntError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub log_filter: String,
    pub db_url: String,

    pub access_token_name: String,
    pub refresh_token_name: String,
    pub cookie_secure: bool,
    pub cookie_domain: Option<String>,
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
        let port = port_raw
            .parse::<u16>()
            .map_err(|source| ConfigError::InvalidNumber {
                key: "SERVER_PORT",
                value: port_raw.clone(),
                source,
            })?;

        let cookie_secure_raw = optional("COOKIE_SECURE", "0")?;
        let cookie_secure = cookie_secure_raw == "1" || cookie_secure_raw == "true";

        Ok(Self {
            host: optional("SERVER_HOST", "0.0.0.0")?,
            port,
            log_filter: optional(
                "LOG_FILTER",
                "info,erp_api=debug,tower_http=debug,sqlx=warn",
            )?,
            access_token_name: required("ACCESS_TOKEN_NAME")?,
            refresh_token_name: required("REFRESH_TOKEN_NAME")?,
            cookie_domain: optional("COOKIE_DOMAIN", "").ok(),
            cookie_secure,
            db_url: required("DB_URL")?,
        })
    }
}
