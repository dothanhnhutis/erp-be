use crate::errors::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "DEACTIVATED")]
    Deactivated,
    #[serde(rename = "PENDING_PASSWORD")]
    PendingPassword,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "ACTIVE",
            UserStatus::Deactivated => "DEACTIVATED",
            UserStatus::PendingPassword => "PENDING_PASSWORD",
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(UserStatus::Active),
            "DEACTIVATED" => Ok(UserStatus::Deactivated),
            "PENDING_PASSWORD" => Ok(UserStatus::PendingPassword),
            other => Err(DomainError::InvalidUserStatus(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub username: Option<String>,
    pub status: UserStatus,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
