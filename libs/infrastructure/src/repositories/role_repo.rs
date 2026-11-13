use domain::repositories::{RepoError, RoleRepo};
use sqlx::{PgPool, types::uuid};

use crate::persistence::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgRoleRepo {
    pool: PgPool,
}

impl PgRoleRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

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

impl RoleRepo for PgRoleRepo {
    async fn find_permission_codes_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<String>, RepoError> {
        let codes: Vec<String> = sqlx::query_scalar(PERMISSION_CODES_FOR_USER)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(codes)
    }
}
