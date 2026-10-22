use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Role đang ACTIVE kèm mã permission của nó.
#[derive(Debug, Clone)]
pub struct RolePermissions {
    pub role_id: Uuid,
    pub codes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: String, // hoặc enum RoleStatus như UserStatus
    pub can_update: bool,
    pub can_delete: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}
