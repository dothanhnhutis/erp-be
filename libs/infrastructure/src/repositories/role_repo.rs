use domain::{
    entities::role::RolePermissions,
    repositories::{RepositoryError, RoleRepo},
};
use sqlx::PgPool;

use crate::persistence::error::map_sqlx_error;

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
}
