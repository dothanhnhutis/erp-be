use chrono::{DateTime, Utc};
use domain::{
    entities::role::{Role, RolePermissions},
    repositories::{RepositoryError, RoleRepo},
};
use sqlx::PgPool;

use crate::persistence::error::map_sqlx_error;

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: uuid::Uuid,
    name: String,
    description: String,
    status: String,
    can_update: bool,
    can_delete: bool,
    deleted_at: Option<DateTime<Utc>>,
}

impl From<RoleRow> for Role {
    fn from(r: RoleRow) -> Self {
        Role {
            id: r.id,
            name: r.name,
            description: r.description,
            status: r.status,
            can_update: r.can_update,
            can_delete: r.can_delete,
            deleted_at: r.deleted_at,
        }
    }
}

const PERMISSION_CODES_FOR_ROLE: &str = r#"
    SELECT p.code
    FROM role_permissions rp
    JOIN permissions p ON p.id = rp.permission_id
    WHERE rp.role_id = $1
    ORDER BY p.code
"#;

const PERMISSION_CODES_FOR_USER: &str = r#"
    SELECT DISTINCT p.code
    FROM user_roles ur
    JOIN role_permissions rp ON rp.role_id = ur.role_id
    JOIN permissions p ON p.id = rp.permission_id
    JOIN roles r ON r.id = ur.role_id
    WHERE ur.user_id = $1
      AND r.status = 'ACTIVE'
      AND r.deleted_at IS NULL
"#;

const ACTIVE_ROLE_PERMISSIONS: &str = r#"
    SELECT r.id,
           COALESCE(array_agg(p.code) FILTER (WHERE p.code IS NOT NULL), '{}') AS codes
    FROM roles r
    LEFT JOIN role_permissions rp ON rp.role_id = r.id
    LEFT JOIN permissions p ON p.id = rp.permission_id
    WHERE r.id = ANY($1)
      AND r.status = 'ACTIVE'
      AND r.deleted_at IS NULL
    GROUP BY r.id
"#;

#[derive(Clone)]
pub struct PgRoleRepo {
    pool: PgPool,
}

impl PgRoleRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RoleRepo for PgRoleRepo {
    async fn find_permission_codes_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<String>, RepositoryError> {
        let codes: Vec<String> = sqlx::query_scalar(PERMISSION_CODES_FOR_USER)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(codes)
    }

    async fn find_active_role_permissions(
        &self,
        role_ids: &[uuid::Uuid],
    ) -> Result<Vec<RolePermissions>, RepositoryError> {
        let rows: Vec<(uuid::Uuid, Vec<String>)> = sqlx::query_as(ACTIVE_ROLE_PERMISSIONS)
            .bind(role_ids)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(rows
            .into_iter()
            .map(|(role_id, codes)| RolePermissions { role_id, codes })
            .collect())
    }

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Role>, RepositoryError> {
        let row: Option<RoleRow> = sqlx::query_as(
            "SELECT id, name, description, status, can_update, can_delete, deleted_at
             FROM roles WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(Role::from))
    }

    async fn find_permission_codes_for_role(
        &self,
        id: uuid::Uuid,
    ) -> Result<Vec<String>, RepositoryError> {
        sqlx::query_scalar(PERMISSION_CODES_FOR_ROLE)
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)
    }

    async fn find_existing_permission_codes(
        &self,
        codes: &[String],
    ) -> Result<Vec<String>, RepositoryError> {
        sqlx::query_scalar("SELECT code FROM permissions WHERE code = ANY($1)")
            .bind(codes)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)
    }

    async fn replace_permissions(
        &self,
        role_id: uuid::Uuid,
        codes: &[String],
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        // Khoá dòng role + chặn lớp 1 ngay ở DB (role hệ thống can_update = FALSE).
        let locked: Option<uuid::Uuid> = sqlx::query_scalar(
            "UPDATE roles SET updated_at = NOW()
             WHERE id = $1 AND deleted_at IS NULL AND can_update
             RETURNING id",
        )
        .bind(role_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;
        if locked.is_none() {
            return Err(RepositoryError::NotFound); // tx drop → rollback
        }

        // Có code không tồn tại → dừng (phòng hờ, service đã kiểm tra).
        let found: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM permissions WHERE code = ANY($1)")
                .bind(codes)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
        if found as usize != codes.len() {
            return Err(RepositoryError::NotFound);
        }

        sqlx::query(
            "DELETE FROM role_permissions rp
             USING permissions p
             WHERE rp.permission_id = p.id AND rp.role_id = $1 AND p.code <> ALL($2)",
        )
        .bind(role_id)
        .bind(codes)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id)
             SELECT $1, p.id FROM permissions p WHERE p.code = ANY($2)
             ON CONFLICT DO NOTHING",
        )
        .bind(role_id)
        .bind(codes)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if inserted as usize != codes.len() {
            return Err(RepositoryError::NotFound);
        }

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }
}
