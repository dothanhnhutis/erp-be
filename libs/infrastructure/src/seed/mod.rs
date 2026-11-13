//! Seed dữ liệu hệ thống (loại 1): chạy ở MỌI môi trường, idempotent.
//! Bootstrap admin và dữ liệu giả nằm ở `apps/seeder`.

pub mod system;

use sqlx::{Postgres, Transaction};

/// Khoá advisory dùng chung cho mọi bước seed, tránh 2 tiến trình (vd 2 replica/job)
/// seed đồng thời. Giá trị bất kỳ nhưng phải cố định.
pub const SEED_LOCK_KEY: i64 = 0x0045_5250_5345_4544; // "ERPSEED"

/// Giá trị `changed_by` trong audit_logs cho mọi thay đổi do seeder tạo ra.
pub const SEED_ACTOR: &str = "SEEDER";

/// Lấy khoá advisory trong phạm vi transaction (tự nhả khi commit/rollback)
/// và đánh dấu người thực hiện cho trigger audit (`ich_app.current_user_id`).
pub async fn lock(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_xact_lock($1), set_config('ich_app.current_user_id', $2, true)")
        .bind(SEED_LOCK_KEY)
        .bind(SEED_ACTOR)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
