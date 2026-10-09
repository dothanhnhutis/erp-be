use chrono::Duration;
use std::sync::Arc;

use application::service::auth_service::{self, AuthService};
use axum::extract::FromRef;
use infrastructure::{
    postgres::pool::init_db_pool,
    repositories::{session_repo::PgSessionRepo, user_repo::PgUserRepo},
};
use shared::config::AppConfig;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub auth_service: Arc<AuthService<PgUserRepo, PgSessionRepo>>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let pool = init_db_pool(&config.database_url)
            .await
            .expect("không kết nối được database");

        let pg_user_repo = PgUserRepo::new(pool.clone());
        let pg_session_repo = PgSessionRepo::new(pool.clone());

        let auth_service = Arc::new(auth_service::AuthService::new(
            pg_user_repo.clone(),
            pg_session_repo.clone(),
            config.jwt_enc.clone(),
            Duration::seconds(config.access_token_ttl_secs),
            Duration::seconds(config.refresh_token_ttl_secs),
            Duration::seconds(config.max_refresh_token_ttl_secs),
        ));

        Self {
            auth_service,
            config: Arc::new(config),
        }
    }
}
