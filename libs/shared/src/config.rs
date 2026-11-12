use jsonwebtoken::{DecodingKey, EncodingKey};
use std::{env, num::ParseIntError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub log_filter: String,
    pub db_url: String,
    // session
    pub access_token_name: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_name: String,
    pub refresh_token_ttl_secs: i64,
    pub max_refresh_token_ttl_secs: i64,
    pub jwt_enc: EncodingKey,
    pub jwt_dec: DecodingKey,
    // cookie
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

        let access_token_ttl_secs_raw = optional("ACCESS_TOKEN_TTL_SECS", "900")?;
        let access_token_ttl_secs = access_token_ttl_secs_raw.parse::<i64>().map_err(|source| {
            ConfigError::InvalidNumber {
                key: "ACCESS_TOKEN_TTL_SECS",
                value: access_token_ttl_secs_raw.clone(),
                source,
            }
        })?;

        let refresh_token_ttl_secs_raw = optional("REFRESH_TOKEN_TTL_SECS", "900")?;
        let refresh_token_ttl_secs =
            refresh_token_ttl_secs_raw
                .parse::<i64>()
                .map_err(|source| ConfigError::InvalidNumber {
                    key: "REFRESH_TOKEN_TTL_SECS",
                    value: refresh_token_ttl_secs_raw.clone(),
                    source,
                })?;

        let max_refresh_token_ttl_secs_raw = optional("MAX_REFRESH_TOKEN_TTL_SECS", "900")?;
        let max_refresh_token_ttl_secs =
            max_refresh_token_ttl_secs_raw
                .parse::<i64>()
                .map_err(|source| ConfigError::InvalidNumber {
                    key: "MAX_REFRESH_TOKEN_TTL_SECS",
                    value: max_refresh_token_ttl_secs_raw.clone(),
                    source,
                })?;

        let jwt_secret: String = required("JWT_SECRET")?;

        Ok(Self {
            host: optional("SERVER_HOST", "0.0.0.0")?,
            port,
            log_filter: optional(
                "LOG_FILTER",
                "info,erp_api=debug,tower_http=debug,sqlx=warn",
            )?,
            access_token_name: required("ACCESS_TOKEN_NAME")?,
            access_token_ttl_secs,
            refresh_token_name: required("REFRESH_TOKEN_NAME")?,
            refresh_token_ttl_secs,
            max_refresh_token_ttl_secs,
            jwt_enc: EncodingKey::from_secret(jwt_secret.as_bytes()),
            jwt_dec: DecodingKey::from_secret(jwt_secret.as_bytes()),
            cookie_domain: optional("COOKIE_DOMAIN", "").ok(),
            cookie_secure,
            db_url: required("DB_URL")?,
        })
    }
}
