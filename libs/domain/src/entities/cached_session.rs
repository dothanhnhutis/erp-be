use serde::{Deserialize, Serialize};

use crate::entities::{session::Session, user::User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSession {
    pub session: Session,
    pub user: User,
}
