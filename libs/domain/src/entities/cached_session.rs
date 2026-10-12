use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entities::{session::Session, user::User};

/// Bản cache của một phiên: gồm session + user + thời điểm cuối cùng đã ghi
/// `expires_at` xuống DB (dùng để throttle ghi DB).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSession {
    pub session: Session,
    pub user: User,
    pub db_synced_at: DateTime<Utc>,
}
