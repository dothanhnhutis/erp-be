//! CLI migrate + seed database.
//!
//! Production (mỗi lần deploy):  erp-seeder all      (= migrate → system → admin)
//! Dev:  cargo run -p erp-seeder --features dev-fixtures -- all
//!       cargo run -p erp-seeder --features dev-fixtures -- dev --reset

mod admin;
#[cfg(feature = "dev-fixtures")]
mod dev;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use infrastructure::{
    postgres::{MIGRATOR, coverage::missing_triggers, pool::create_db_pool},
    seed::system::sync_system,
};
use shared::config::{AppEnv, DATABASE_URL, database_url};
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "erp-seeder", about = "Migrate và seed database ERP")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Chạy các migration chưa áp dụng
    Migrate {
        /// Revert migration gần nhất (cấm khi APP_ENV=production)
        #[arg(long)]
        revert: bool,
    },
    /// Đồng bộ permission + role hệ thống (idempotent, chạy mỗi lần deploy)
    System,
    /// Tạo super admin đầu tiên nếu chưa có (SEED_ADMIN_EMAIL / SEED_ADMIN_PASSWORD)
    Admin,
    /// Dữ liệu giả cho dev (cấm khi APP_ENV=production)
    #[cfg(feature = "dev-fixtures")]
    Dev {
        /// Xoá sạch dữ liệu (TRUNCATE) rồi seed lại từ đầu
        #[arg(long)]
        reset: bool,
        /// Cho phép chạy khi DATABASE_URL không trỏ tới localhost
        #[arg(long)]
        force: bool,
    },
    /// migrate → system → admin (→ dev nếu build kèm feature dev-fixtures và không phải production)
    All,
}

#[tokio::main]
async fn main() -> Result<()> {
    // .env là tuỳ chọn: ở production biến môi trường đến từ secret store / orchestrator.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("SEEDER_LOG")
                .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let env = AppEnv::from_env()?;
    let db_url = database_url()?;
    let pool = create_db_pool(&db_url)
        .await
        .with_context(|| format!("không kết nối được database ({DATABASE_URL})"))?;

    tracing::info!(app_env = %env, "erp-seeder bắt đầu");

    match cli.command {
        Command::Migrate { revert: false } => migrate(&pool).await?,
        Command::Migrate { revert: true } => revert_last(&pool, env).await?,
        Command::System => system(&pool).await?,
        Command::Admin => admin::bootstrap_admin(&pool, env).await?,
        #[cfg(feature = "dev-fixtures")]
        Command::Dev { reset, force } => {
            dev::guard(env, &db_url, force)?;
            if reset {
                dev::reset(&pool).await?;
                system(&pool).await?;
                admin::bootstrap_admin(&pool, env).await?;
            }
            dev::seed_fixtures(&pool).await?;
        }
        Command::All => {
            migrate(&pool).await?;
            system(&pool).await?;
            admin::bootstrap_admin(&pool, env).await?;
            #[cfg(feature = "dev-fixtures")]
            if env.is_production() {
                tracing::info!("APP_ENV=production → bỏ qua dữ liệu giả");
            } else {
                dev::guard(env, &db_url, false)?;
                dev::seed_fixtures(&pool).await?;
            }
        }
    }

    tracing::info!("hoàn tất");
    Ok(())
}

async fn migrate(pool: &PgPool) -> Result<()> {
    MIGRATOR.run(pool).await.context("chạy migration thất bại")?;
    tracing::info!("migration: đã cập nhật tới bản mới nhất");
    warn_missing_triggers(pool).await
}

/// Cảnh báo bảng thiếu trigger audit / updated_at (migration quên gọi sync_*).
/// Chỉ cảnh báo, không tự sửa: trigger là schema, phải nằm trong migration.
async fn warn_missing_triggers(pool: &PgPool) -> Result<()> {
    let missing = missing_triggers(pool)
        .await
        .context("không kiểm tra được trigger của các bảng")?;
    for m in &missing {
        let fix = match m.kind.as_str() {
            "audit" => {
                "thêm `SELECT sync_audit_triggers();` vào cuối migration tạo bảng \
                 (hoặc INSERT INTO audit_excluded_tables nếu cố ý không audit)"
            }
            _ => "thêm `SELECT sync_updated_at_triggers();` vào cuối migration tạo bảng",
        };
        tracing::warn!(table = %m.table_name, kind = %m.kind, "bảng thiếu trigger — {fix}");
    }
    Ok(())
}

async fn revert_last(pool: &PgPool, env: AppEnv) -> Result<()> {
    if env.is_production() {
        bail!("không cho phép revert migration khi APP_ENV=production — viết migration mới để sửa");
    }

    let applied: Vec<i64> = sqlx::query_scalar(
        "SELECT version FROM _sqlx_migrations WHERE success ORDER BY version DESC LIMIT 2",
    )
    .fetch_all(pool)
    .await
    .context("không đọc được bảng _sqlx_migrations")?;

    // undo(target) revert mọi migration có version > target.
    let (last, target) = match applied.as_slice() {
        [] => {
            tracing::info!("chưa có migration nào để revert");
            return Ok(());
        }
        [last] => (*last, 0),
        [last, prev, ..] => (*last, *prev),
    };

    MIGRATOR
        .undo(pool, target)
        .await
        .context("revert migration thất bại")?;
    tracing::info!(version = last, "đã revert migration");
    Ok(())
}

async fn system(pool: &PgPool) -> Result<()> {
    let report = sync_system(pool)
        .await
        .context("đồng bộ dữ liệu hệ thống thất bại (đã chạy `migrate` chưa?)")?;
    tracing::info!(
        permissions_changed = report.permissions_changed,
        orphan_permissions = report.orphan_permissions.len(),
        roles_created = report.roles_created,
        grants_added = report.grants_added,
        "system: đã đồng bộ permission + role hệ thống"
    );
    Ok(())
}
