// libs/infrastructure/src/bin/postgres_seed.rs
//
// Tạo tài khoản admin đầu tiên. Mật khẩu lấy từ biến môi trường, KHÔNG nằm trong git.
// Chạy lại nhiều lần an toàn: email đã tồn tại thì giữ nguyên mật khẩu cũ, chỉ đảm bảo có vai trò ADMIN.
//
// Chạy: ADMIN_EMAIL=admin@company.vn ADMIN_PASSWORD='...' cargo run -p infrastructure --bin postgres_seed

use argon2::{Argon2, password_hash::PasswordHasher};
use std::env;
use tokio::task;

/// Trùng với tên vai trò trong migration default_roles_permissions
const ADMIN_ROLE_NAME: &str = "ADMIN";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let pool = sqlx::PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let email = env::var("ADMIN_EMAIL")
        .map_err(|_| "Thiếu biến ADMIN_EMAIL")?
        .trim()
        .to_lowercase();
    let password = env::var("ADMIN_PASSWORD").map_err(|_| "Thiếu biến ADMIN_PASSWORD")?;

    if password.chars().count() < 12 {
        return Err("ADMIN_PASSWORD phải có ít nhất 12 ký tự".into());
    }

    // Phải dùng CÙNG thuật toán với phần verify mật khẩu lúc đăng nhập trong apps/api
    let password_hash: String = task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string()) // PasswordHash -> chuỗi PHC để lưu DB
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??; // `?` ngoài: JoinError, `?` trong: lỗi hash

    let mut tx = pool.begin().await?;

    // id của vai trò do database tự sinh, nên tra theo tên
    let role_id: Option<String> =
        sqlx::query_scalar("SELECT id::text FROM roles WHERE name = $1 AND deleted_at IS NULL")
            .bind(ADMIN_ROLE_NAME)
            .fetch_optional(&mut *tx)
            .await?;
    let role_id =
        role_id.ok_or("Chưa có vai trò ADMIN. Hãy chạy `sqlx migrate run` trước khi seed.")?;

    // Khớp với unique index idx_users_email_unique (lower(email)) WHERE deleted_at IS NULL
    let created = sqlx::query(
        "INSERT INTO users (email, username, password_hash, status, password_changed_at)
         VALUES ($1, 'admin', $2, 'ACTIVE', NOW())
         ON CONFLICT (lower(email)) WHERE deleted_at IS NULL DO NOTHING",
    )
    .bind(&email)
    .bind(&password_hash)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    let user_id: String =
        sqlx::query_scalar("SELECT id::text FROM users WHERE email = $1 AND deleted_at IS NULL")
            .bind(&email)
            .fetch_one(&mut *tx)
            .await?;

    // Khôi phục role ADMIN: ACTIVE, không bị xoá, khoá sửa/xoá, có MỌI permission.
    sqlx::query(
        "UPDATE roles
     SET status = 'ACTIVE', deactivated_at = NULL, deleted_at = NULL,
         can_update = FALSE, can_delete = FALSE
     WHERE id = $1::uuid",
    )
    .bind(&role_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO role_permissions (role_id, permission_id)
     SELECT $1::uuid, p.id FROM permissions p
     ON CONFLICT DO NOTHING",
    )
    .bind(&role_id)
    .execute(&mut *tx)
    .await?;

    // Mở lại tài khoản admin nếu bị khoá (chỉ khi đã có mật khẩu — thoả chk_users_pending_password).
    sqlx::query(
        "UPDATE users SET status = 'ACTIVE', deactivated_at = NULL
     WHERE id = $1::uuid AND password_hash IS NOT NULL",
    )
    .bind(&user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    let redis_url = env::var("REDIS_URL")?;
    let mut conn = infrastructure::cache::connect::init_redis(&redis_url).await?;
    let _: u64 = redis::AsyncCommands::incr(&mut conn, "perm:ver", 1).await?;
    println!("Đã khôi phục quyền ADMIN và làm mới cache permission");

    if created == 1 {
        println!("Đã tạo admin {email} (id {user_id})");
    } else {
        println!("Admin {email} đã tồn tại, giữ nguyên mật khẩu, đã đảm bảo vai trò ADMIN");
    }
    Ok(())
}
