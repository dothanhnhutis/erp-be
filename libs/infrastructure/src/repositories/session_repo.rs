use domain::{
    entities::session::{DeviceType, NewSession, Session},
    errors::DomainError,
    repositories::{RepositoryError, SessionRepo},
};
use sqlx::{
    PgPool,
    types::chrono::{DateTime, Utc},
};
use std::str::FromStr;

use crate::persistence::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgSessionRepo {
    pool: PgPool,
}

impl PgSessionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Struct riêng cho DB layer — tách biệt khỏi domain entity.
/// `ip_address` lấy về dạng text (`ip_address::text`) vì cột là kiểu INET.
#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    refresh_token_hash: String,
    previous_token_hash: Option<String>,
    rotated_at: Option<DateTime<Utc>>,
    device_name: Option<String>,
    device_type: String,
    app_version: Option<String>,
    user_agent: Option<String>,
    ip_address: Option<String>,
    revoked_at: Option<DateTime<Utc>>,
    revoke_reason: Option<String>,
    expires_at: DateTime<Utc>,
    absolute_expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Mapping từ DB row → Domain entity (1:1, không có field nào cần convert).
impl TryFrom<SessionRow> for Session {
    type Error = DomainError;
    fn try_from(row: SessionRow) -> Result<Self, Self::Error> {
        Ok(Session {
            id: row.id,
            user_id: row.user_id,
            refresh_token_hash: row.refresh_token_hash,
            previous_token_hash: row.previous_token_hash,
            rotated_at: row.rotated_at,
            device_name: row.device_name,
            device_type: DeviceType::from_str(&row.device_type)?,
            app_version: row.app_version,
            user_agent: row.user_agent,
            ip_address: row.ip_address,
            revoked_at: row.revoked_at,
            revoke_reason: row.revoke_reason,
            expires_at: row.expires_at,
            absolute_expires_at: row.absolute_expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

const INSERT_SESSION: &str = r#"
    INSERT INTO user_sessions
        (user_id, refresh_token_hash, device_name, device_type, app_version,
        user_agent, ip_address, expires_at, absolute_expires_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7::inet, $8, $9)
    RETURNING
        id, user_id, refresh_token_hash, previous_token_hash, rotated_at, device_name, device_type,
        app_version, user_agent, ip_address::text AS ip_address,
        revoked_at, revoke_reason, expires_at, absolute_expires_at, created_at, updated_at
"#;

const SELECT_SESSION_BY_TOKEN: &str = r#"
    SELECT
        id, user_id, token_hash, device_id, device_name, device_type,
        platform, app_version, user_agent,
        ip_address::text AS ip_address,
        revoked_at, revoke_reason, expires_at, created_at, updated_at
    FROM user_sessions
    WHERE token_hash = $1
"#;

impl SessionRepo for PgSessionRepo {
    async fn create(&self, new_session: NewSession) -> Result<Session, RepositoryError> {
        let row: SessionRow = sqlx::query_as(INSERT_SESSION)
            .bind(new_session.user_id)
            .bind(new_session.refresh_token_hash)
            .bind(new_session.device_name)
            .bind(new_session.device_type.as_str())
            .bind(new_session.app_version)
            .bind(new_session.user_agent)
            .bind(new_session.ip_address)
            .bind(new_session.expires_at)
            .bind(new_session.absolute_expires_at)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        // Ok(row.try_into())
        Ok(Session::try_from(row)?)
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, RepositoryError> {
        let row: Option<SessionRow> = sqlx::query_as(SELECT_SESSION_BY_TOKEN)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(row.map(Session::from))
    }
}
