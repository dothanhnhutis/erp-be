use std::sync::Arc;

use application::services::auth_service::AuthService;
use axum::extract::FromRef;
use domain::repositories::UserRepo;
use infrastructure::repositories::{
    session_repo::{self, PgSessionRepo},
    user_repo::PgUserRepo,
};
use shared::config::AppConfig;
use sqlx::PgPool;

pub struct StateData {
    pub config: AppConfig,
    pub pool: PgPool,
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_service: Arc<AuthService<PgUserRepo, PgSessionRepo>>,
}

impl AppState {
    pub fn new(data: StateData) -> Self {
        let user_repo = PgUserRepo::new(data.pool.clone());
        let session_repo = PgSessionRepo::new(data.pool.clone());

        let auth_service = Arc::new(AuthService::new(user_repo, session_repo));

        Self {
            config: Arc::new(data.config),
            auth_service,
        }
    }
}
