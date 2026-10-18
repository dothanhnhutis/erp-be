use std::sync::Arc;

use application::service::auth_service::{self, AuthService};
use axum::extract::FromRef;
use infrastructure::{
    cache::{connect::init_redis, permission::RedisPermissionCache, session::RedisSessionCache},
    postgres::pool::init_db_pool,
    repositories::{
        role_repo::PgRoleRepository, session_repo::PgSessionRepo, user_repo::PgUserRepo,
    },
};
use shared::config::{AppConfig, TokenConfig};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub auth_service: Arc<
        AuthService<
            PgUserRepo,
            PgSessionRepo,
            PgRoleRepository,
            RedisSessionCache,
            RedisPermissionCache,
        >,
    >,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let pool = init_db_pool(&config.database_url)
            .await
            .expect("không kết nối được database");

        let redis_conn = init_redis(&config.redis_url)
            .await
            .expect("Failed to connect to Redis");

        let pg_user_repo = PgUserRepo::new(pool.clone());
        let pg_session_repo = PgSessionRepo::new(pool.clone());
        let pg_role_repo = PgRoleRepository::new(pool.clone());

        let session_cache = RedisSessionCache::new(redis_conn.clone());
        let per_cache = RedisPermissionCache::new(redis_conn.clone());

        let token = TokenConfig::from(&config);
        let auth_service = Arc::new(auth_service::AuthService::new(
            pg_user_repo.clone(),
            pg_session_repo.clone(),
            pg_role_repo.clone(),
            session_cache,
            per_cache,
            token,
        ));

        Self {
            auth_service,
            config: Arc::new(config),
        }
    }
}
