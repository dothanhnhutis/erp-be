//! Dữ liệu giả cho dev (chỉ biên dịch khi bật feature `dev-fixtures`).
//! Sinh theo chỉ số → mọi máy dev có cùng dữ liệu; chạy lại không tạo trùng.

use anyhow::{Context, Result, bail};
use application::security::password::hash_password;
use domain::permissions::codes;
use infrastructure::seed;
use shared::config::AppEnv;
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::str::FromStr;
use uuid::Uuid;

/// Mật khẩu chung của mọi user giả.
pub const DEV_PASSWORD: &str = "Password@123";
const USER_COUNT: usize = 20;

struct SampleRole {
    name: &'static str,
    description: &'static str,
    permissions: &'static [&'static str],
}

const SAMPLE_ROLES: &[SampleRole] = &[
    SampleRole {
        name: "Nhân sự",
        description: "Quản lý tài khoản nhân viên",
        permissions: &[
            codes::USER_VIEW,
            codes::USER_CREATE,
            codes::USER_UPDATE,
            codes::USER_DEACTIVATE,
        ],
    },
    SampleRole {
        name: "Kế toán",
        description: "Phòng kế toán",
        permissions: &[codes::USER_VIEW],
    },
    SampleRole {
        name: "Thủ kho",
        description: "Quản lý kho",
        permissions: &[],
    },
];

/// Chặn chạy nhầm vào DB thật.
pub fn guard(env: AppEnv, db_url: &str, force: bool) -> Result<()> {
    if env.is_production() {
        bail!("từ chối seed dữ liệu giả khi APP_ENV=production");
    }
    let opts = PgConnectOptions::from_str(db_url).context("DATABASE_URL không hợp lệ")?;
    let host = opts.get_host();
    let is_local = matches!(host, "localhost" | "127.0.0.1" | "::1") || host.starts_with('/');
    if !is_local && !force {
        bail!("DATABASE_URL trỏ tới host `{host}` (không phải localhost) — thêm --force nếu chắc chắn");
    }
    Ok(())
}

/// Xoá sạch dữ liệu (giữ schema). TRUNCATE không kích hoạt trigger bảo vệ role hệ thống.
pub async fn reset(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        TRUNCATE user_avatars, password_tokens, user_sessions, user_roles,
                 role_permissions, files, users, roles, permissions, audit_logs
        RESTART IDENTITY CASCADE
        "#,
    )
    .execute(pool)
    .await
    .context("TRUNCATE thất bại")?;
    tracing::info!("dev: đã xoá sạch dữ liệu");
    Ok(())
}

pub async fn seed_fixtures(pool: &PgPool) -> Result<()> {
    // argon2 chậm → hash 1 lần, dùng cho mọi user.
    let password_hash = tokio::task::spawn_blocking(|| hash_password(DEV_PASSWORD))
        .await
        .context("tác vụ hash mật khẩu bị huỷ")??;

    let mut tx = pool.begin().await?;
    seed::lock(&mut tx).await?;

    let mut role_ids = Vec::with_capacity(SAMPLE_ROLES.len());
    for role in SAMPLE_ROLES {
        // uq_roles_name là partial unique index (deleted_at IS NULL) → phải ghi rõ WHERE.
        let id: Uuid = sqlx::query_scalar(
            r#"
            WITH ins AS (
                INSERT INTO roles (name, description)
                VALUES ($1, $2)
                ON CONFLICT (name) WHERE deleted_at IS NULL DO NOTHING
                RETURNING id
            )
            SELECT id FROM ins
            UNION ALL
            SELECT id FROM roles WHERE name = $1 AND deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(role.name)
        .bind(role.description)
        .fetch_one(&mut *tx)
        .await?;

        let codes: Vec<String> = role.permissions.iter().map(|c| (*c).to_owned()).collect();
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            SELECT $1, id FROM permissions WHERE code = ANY($2::text[])
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(id)
        .bind(&codes)
        .execute(&mut *tx)
        .await?;

        role_ids.push(id);
    }

    let mut created = 0usize;
    for i in 1..=USER_COUNT {
        let email = format!("user{i:02}@erp.local");
        let username = format!("user{i:02}");
        // Đủ các trạng thái để test nhánh lỗi của login.
        let (status, hash): (&str, Option<&str>) = match i % 10 {
            9 => ("DEACTIVATED", Some(&password_hash)),
            0 => ("PENDING_PASSWORD", None),
            _ => ("ACTIVE", Some(&password_hash)),
        };

        let user_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            INSERT INTO users (email, password_hash, username, status, deactivated_at, password_changed_at)
            VALUES (
                $1, $2, $3, $4,
                CASE WHEN $4 = 'DEACTIVATED' THEN NOW() END,
                CASE WHEN $2::text IS NOT NULL THEN NOW() END
            )
            ON CONFLICT (lower(email)) WHERE deleted_at IS NULL DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&email)
        .bind(hash)
        .bind(&username)
        .bind(status)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(user_id) = user_id {
            created += 1;
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(role_ids[i % role_ids.len()])
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    tracing::info!(
        roles = SAMPLE_ROLES.len(),
        users_created = created,
        users_total = USER_COUNT,
        password = DEV_PASSWORD,
        "dev: đã seed dữ liệu giả (user01..user{USER_COUNT:02}@erp.local)"
    );
    Ok(())
}
