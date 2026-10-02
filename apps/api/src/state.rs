use infrastructure::postgres::pool::init_db_pool;
use shared::config::AppConfig;

struct AppState {}

pub async fn init_state(config: &AppConfig) -> anyhow::Result<()> {
    let pool = init_db_pool(&config.database_url)
        .await
        .expect("không kết nối được database");

    Ok(())
}
