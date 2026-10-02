use axum::extract::FromRef;
use infrastructure::postgres::pool::init_db_pool;
use shared::config::AppConfig;

#[derive(Clone, FromRef)]
pub struct AppState {}

pub async fn init_state(config: &AppConfig) -> AppState {
    let pool = init_db_pool(&config.database_url)
        .await
        .expect("không kết nối được database");

    let state = AppState {};
    state
}
