pub mod coverage;
pub mod pool;

/// Bộ migration của database chính, nhúng sẵn vào binary.
/// Đường dẫn tính từ thư mục gốc crate `infrastructure`.
/// Dùng chung cho `erp-seeder migrate`, api (dev) và `#[sqlx::test(migrator = "...")]`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
