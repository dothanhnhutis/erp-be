//! Chặn migration quên `SELECT sync_updated_at_triggers(); SELECT sync_audit_triggers();`.
//! Chạy: cargo test -p infrastructure --test trigger_coverage  (cần DATABASE_URL, user có quyền CREATE DATABASE)

use infrastructure::postgres::coverage::missing_triggers;
use sqlx::PgPool;

#[sqlx::test(migrator = "infrastructure::postgres::MIGRATOR")]
async fn every_table_has_required_triggers(pool: PgPool) {
    let missing = missing_triggers(&pool).await.expect("truy vấn coverage");
    assert!(
        missing.is_empty(),
        "các bảng thiếu trigger (thêm sync_*() vào cuối migration, hoặc audit_excluded_tables): {missing:#?}"
    );
}

#[sqlx::test(migrator = "infrastructure::postgres::MIGRATOR")]
async fn new_table_is_detected_then_fixed_by_sync(pool: PgPool) {
    sqlx::query("CREATE TABLE demo_cov (id uuid PRIMARY KEY DEFAULT uuidv7(), updated_at timestamptz)")
        .execute(&pool)
        .await
        .unwrap();

    let kinds: Vec<String> = missing_triggers(&pool)
        .await
        .unwrap()
        .into_iter()
        .filter(|m| m.table_name == "demo_cov")
        .map(|m| m.kind)
        .collect();
    assert_eq!(kinds, ["audit", "updated_at"]);

    sqlx::query("SELECT sync_updated_at_triggers(), sync_audit_triggers()")
        .execute(&pool)
        .await
        .unwrap();
    assert!(missing_triggers(&pool).await.unwrap().is_empty());
}

#[sqlx::test(migrator = "infrastructure::postgres::MIGRATOR")]
async fn table_without_primary_key_must_be_excluded(pool: PgPool) {
    sqlx::query("CREATE TABLE demo_nopk (v int)")
        .execute(&pool)
        .await
        .unwrap();

    let err = sqlx::query("SELECT sync_audit_triggers()")
        .execute(&pool)
        .await
        .expect_err("bảng không có khoá chính phải bị từ chối");
    assert!(err.to_string().contains("demo_nopk"), "{err}");

    sqlx::query("INSERT INTO audit_excluded_tables (table_name, reason) VALUES ('demo_nopk', 'test')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("SELECT sync_audit_triggers()")
        .execute(&pool)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "infrastructure::postgres::MIGRATOR")]
async fn audit_masks_secrets_and_skips_noop_updates(pool: PgPool) {
    sqlx::query(
        "INSERT INTO users (email, password_hash, status) VALUES ('a@erp.local', 'phc-secret', 'ACTIVE')",
    )
    .execute(&pool)
    .await
    .unwrap();
    // UPDATE không đổi dữ liệu → không ghi log
    sqlx::query("UPDATE users SET status = status").execute(&pool).await.unwrap();
    sqlx::query("UPDATE users SET username = 'a', password_hash = 'phc-new'")
        .execute(&pool)
        .await
        .unwrap();

    let rows: Vec<(String, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT action, new_data FROM audit_logs WHERE table_name = 'users' ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(rows.len(), 2, "{rows:#?}");
    assert_eq!(rows[0].0, "INSERT");
    assert_eq!(rows[1].0, "UPDATE");
    let upd = rows[1].1.as_ref().unwrap();
    assert_eq!(upd["password_hash"], "***REDACTED***");
    assert_eq!(upd["username"], "a");
    assert!(upd.get("email").is_none(), "UPDATE chỉ ghi cột thay đổi: {upd}");
}
