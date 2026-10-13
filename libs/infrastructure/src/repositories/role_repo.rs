use domain::repositories::{RepositoryError, RoleRepository};
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

#[derive(Clone)]
pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RoleRepository for PgRoleRepository {
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
}
