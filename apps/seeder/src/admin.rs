//! Bootstrap super admin đầu tiên. Chỉ tạo khi CHƯA có user nào giữ role SUPER_ADMIN,
//! nên chạy lại mỗi lần deploy là no-op (và không cần SEED_ADMIN_PASSWORD nữa).

use anyhow::{Context, Result, bail};
use application::security::password::hash_password;
use domain::permissions::SUPER_ADMIN;
use infrastructure::seed;
use shared::config::AppEnv;
use sqlx::PgPool;
use uuid::Uuid;

const DEV_DEFAULT_EMAIL: &str = "admin@erp.local";
const DEV_DEFAULT_PASSWORD: &str = "Admin@123";
const PROD_MIN_PASSWORD_LEN: usize = 12;

pub async fn bootstrap_admin(pool: &PgPool, env: AppEnv) -> Result<()> {
    let mut tx = pool.begin().await?;
    seed::lock(&mut tx).await?;

    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM user_roles ur
            JOIN roles r ON r.id = ur.role_id
            JOIN users u ON u.id = ur.user_id
            WHERE r.code = $1 AND u.deleted_at IS NULL
        )
        "#,
    )
    .bind(SUPER_ADMIN.code)
    .fetch_one(&mut *tx)
    .await?;

    if exists {
        tracing::info!("admin: đã có super admin → bỏ qua");
        return Ok(());
    }

    let role_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM roles WHERE code = $1")
        .bind(SUPER_ADMIN.code)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(role_id) = role_id else {
        bail!(
            "chưa có role {}: chạy `erp-seeder system` trước",
            SUPER_ADMIN.code
        );
    };

    // Chỉ đọc credential khi thực sự cần tạo → sau lần đầu có thể xoá secret.
    let (email, password) = read_credentials(env)?;

    let taken: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM users WHERE lower(email) = $1 AND deleted_at IS NULL)",
    )
    .bind(&email)
    .fetch_one(&mut *tx)
    .await?;
    if taken {
        bail!(
            "email {email} đã thuộc một user khác — dùng email khác cho SEED_ADMIN_EMAIL \
             hoặc gán role {} cho user đó thủ công",
            SUPER_ADMIN.code
        );
    }

    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .context("tác vụ hash mật khẩu bị huỷ")??;

    let user_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (email, password_hash, status, password_changed_at)
        VALUES ($1, $2, 'ACTIVE', NOW())
        RETURNING id
        "#,
    )
    .bind(&email)
    .bind(&password_hash)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(role_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    tracing::info!(%email, "admin: đã tạo super admin");
    if env.is_production() {
        tracing::warn!("hãy xoá SEED_ADMIN_PASSWORD khỏi secret store — không còn cần nữa");
    }
    Ok(())
}

fn read_credentials(env: AppEnv) -> Result<(String, String)> {
    let email = match non_empty_var("SEED_ADMIN_EMAIL") {
        Some(v) => v,
        None if !env.is_production() => DEV_DEFAULT_EMAIL.to_owned(),
        None => bail!("thiếu SEED_ADMIN_EMAIL (bắt buộc khi APP_ENV=production)"),
    };
    // users.email phải lưu lowercase: login so khớp theo lower(email).
    let email = email.trim().to_lowercase();
    if !is_plausible_email(&email) {
        bail!("SEED_ADMIN_EMAIL không hợp lệ: {email}");
    }

    let password = match non_empty_var("SEED_ADMIN_PASSWORD") {
        Some(v) => v,
        None if !env.is_production() => {
            tracing::warn!("dùng mật khẩu admin mặc định của dev: {DEV_DEFAULT_PASSWORD}");
            DEV_DEFAULT_PASSWORD.to_owned()
        }
        None => bail!("thiếu SEED_ADMIN_PASSWORD (bắt buộc khi APP_ENV=production)"),
    };
    if env.is_production() && password.chars().count() < PROD_MIN_PASSWORD_LEN {
        bail!("SEED_ADMIN_PASSWORD phải dài ít nhất {PROD_MIN_PASSWORD_LEN} ký tự ở production");
    }

    Ok((email, password))
}

fn non_empty_var(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

fn is_plausible_email(email: &str) -> bool {
    match email.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !domain.contains('@')
                && !email.contains(char::is_whitespace)
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_plausible_email;

    #[test]
    fn email_check() {
        assert!(is_plausible_email("admin@erp.local"));
        assert!(!is_plausible_email("admin"));
        assert!(!is_plausible_email("admin@local"));
        assert!(!is_plausible_email("a b@erp.vn"));
        assert!(!is_plausible_email("a@b@erp.vn"));
    }
}
