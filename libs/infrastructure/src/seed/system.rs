use domain::permissions::{self, SystemRoleDef};
use sqlx::{PgPool, Postgres, Transaction, types::uuid::Uuid};

#[derive(Debug, Default)]
pub struct SyncReport {
    /// Số permission được thêm mới hoặc đổi mô tả.
    pub permissions_changed: u64,
    /// Permission có trong DB nhưng không còn trong code — KHÔNG tự xoá, chỉ báo.
    pub orphan_permissions: Vec<String>,
    /// Số role hệ thống được tạo mới.
    pub roles_created: u64,
    /// Số cặp role_permissions được gán thêm.
    pub grants_added: u64,
}

/// Đồng bộ danh mục permission + role hệ thống từ `domain::permissions` xuống DB.
/// Idempotent, chạy trong 1 transaction, an toàn khi chạy lại mỗi lần deploy.
pub async fn sync_system(pool: &PgPool) -> Result<SyncReport, sqlx::Error> {
    let mut tx = pool.begin().await?;
    super::lock(&mut tx).await?;

    let mut report = SyncReport::default();

    // 1. Permission: thêm mới / cập nhật mô tả.
    let codes: Vec<String> = permissions::ALL.iter().map(|p| p.code.to_owned()).collect();
    let descriptions: Vec<String> = permissions::ALL
        .iter()
        .map(|p| p.description.to_owned())
        .collect();

    report.permissions_changed = sqlx::query(
        r#"
        INSERT INTO permissions (code, description)
        SELECT * FROM UNNEST($1::text[], $2::text[])
        ON CONFLICT (code) DO UPDATE
            SET description = EXCLUDED.description
            WHERE permissions.description IS DISTINCT FROM EXCLUDED.description
        "#,
    )
    .bind(&codes)
    .bind(&descriptions)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    report.orphan_permissions = sqlx::query_scalar(
        "SELECT code FROM permissions WHERE code <> ALL($1::text[]) ORDER BY code",
    )
    .bind(&codes)
    .fetch_all(&mut *tx)
    .await?;

    for code in &report.orphan_permissions {
        tracing::warn!(
            permission = %code,
            "permission có trong DB nhưng không còn trong code; không tự xoá — dọn tay nếu chắc chắn"
        );
    }

    // 2. Role hệ thống.
    for role in permissions::SYSTEM_ROLES {
        let (role_id, created) = upsert_system_role(&mut tx, role).await?;
        if created {
            report.roles_created += 1;
        }

        if role.grant_all_permissions {
            report.grants_added += sqlx::query(
                r#"
                INSERT INTO role_permissions (role_id, permission_id)
                SELECT $1, id FROM permissions
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(role_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        }
    }

    tx.commit().await?;
    Ok(report)
}

/// Tạo role hệ thống nếu chưa có (theo `code`). Không ghi đè `name`/`description`
/// vì admin có thể đã đổi trên UI. Trả về (id, có_tạo_mới).
async fn upsert_system_role(
    tx: &mut Transaction<'_, Postgres>,
    role: &SystemRoleDef,
) -> Result<(Uuid, bool), sqlx::Error> {
    let inserted: Option<Uuid> = sqlx::query_scalar(
        r#"
        INSERT INTO roles (code, name, description)
        VALUES ($1, $2, $3)
        ON CONFLICT (code) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(role.code)
    .bind(role.default_name)
    .bind(role.description)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(id) = inserted {
        return Ok((id, true));
    }

    let id = sqlx::query_scalar("SELECT id FROM roles WHERE code = $1")
        .bind(role.code)
        .fetch_one(&mut **tx)
        .await?;
    Ok((id, false))
}

/// Lấy id của role hệ thống theo code.
pub async fn system_role_id(pool: &PgPool, code: &str) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM roles WHERE code = $1")
        .bind(code)
        .fetch_optional(pool)
        .await
}
