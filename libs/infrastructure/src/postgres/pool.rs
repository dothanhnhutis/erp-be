use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_db_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(db_url)
        .await?;

    tracing::info!("connect to database success.");

    Ok(pool)
}
