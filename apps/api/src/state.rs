use axum::extract::FromRef;
use infrastructure::{postgres::pool::init_db_pool, repositories::user_repo::PgUserRepo};
use shared::config::AppConfig;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pg_user_repo: PgUserRepo,
}

pub async fn init_state(config: &AppConfig) -> AppState {
    let pool = init_db_pool(&config.database_url)
        .await
        .expect("không kết nối được database");

    let pg_user_repo = PgUserRepo::new(pool.clone());

    AppState { pg_user_repo }
}
