use crate::persistence::error::map_sqlx_error;
use domain::{
    entities::user::{CreateUserWithRoles, User, UserStatus},
    errors::DomainError,
    repositories::{RepositoryError, UserRepo},
};
use sqlx::{
    AssertSqlSafe, PgPool,
    types::chrono::{DateTime, Utc},
};
use std::str::FromStr;
use uuid;

#[derive(Clone)]
pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    password_hash: Option<String>,
    username: Option<String>,
    status: String,
    deactivated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Mapping từ DB row → Domain entity
impl TryFrom<UserRow> for User {
    type Error = DomainError;
    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            status: UserStatus::from_str(&row.status)?,
            deactivated_at: row.deactivated_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

const SELECT_USER: &str = r#"
    SELECT id, email, password_hash, username, status,
           deactivated_at, created_at, updated_at
    FROM users
"#;

impl UserRepo for PgUserRepo {
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        let sql = format!("{} WHERE id = $1 AND deleted_at IS NULL", SELECT_USER);
        let row: Option<UserRow> = sqlx::query_as(AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(row.map(User::try_from).transpose()?)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let sql = format!(
            "{} WHERE lower(email) = lower($1) AND deleted_at IS NULL",
            SELECT_USER
        );
        let row: Option<UserRow> = sqlx::query_as(AssertSqlSafe(sql))
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(row.map(User::try_from).transpose()?) // (4) ? tự đổi DomainError -> Mapping
    }

    async fn create_with_roles(
        &self,
        payload: CreateUserWithRoles,
    ) -> Result<User, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        // Tạo user — status mặc định PENDING_PASSWORD, id do DB sinh.
        let row: UserRow = sqlx::query_as(
            r#"
            INSERT INTO users (email)
            VALUES (lower($1))
            RETURNING id, email, password_hash, username, status,
                      deactivated_at, created_at, updated_at
            "#,
        )
        .bind(&payload.email)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?; // unique_violation → "Email đã tồn tại"

        let inserted = sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            SELECT $1, r.id FROM roles r
            WHERE r.id = ANY($2) AND r.deleted_at IS NULL AND r.status = 'ACTIVE'
            "#,
        )
        .bind(row.id)
        .bind(&payload.role_ids) // service đã sort + dedup
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        if inserted as usize != payload.role_ids.len() {
            return Err(RepositoryError::NotFound); // hoặc variant riêng → "Vai trò không tồn tại hoặc đã bị vô hiệu"
        }

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(User::try_from(row)?)
    }
}
