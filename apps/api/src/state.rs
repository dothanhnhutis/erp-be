use std::sync::Arc;

use application::services::auth_service::{AuthService, AuthServiceInit};
use axum::extract::FromRef;

use infrastructure::{
    cache::{connect::init_redis, permission::RedisPermissionCache, session::RedisSessionCache},
    postgres::pool::create_db_pool,
    repositories::{role_repo::PgRoleRepo, session_repo::PgSessionRepo, user_repo::PgUserRepo},
};
use shared::config::{AppConfig, SessionConfig, TokenConfig};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_service: Arc<
        AuthService<PgUserRepo, PgSessionRepo, PgRoleRepo, RedisSessionCache, RedisPermissionCache>,
    >,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Self {
        let pg_pool = create_db_pool(&config.db_url)
            .await
            .expect("không kết nối được database");

        let redis_conn = init_redis(&config.redis_url)
            .await
            .expect("Failed to connect to Redis");

        let user_repo = PgUserRepo::new(pg_pool.clone());
        let session_repo = PgSessionRepo::new(pg_pool.clone());
        let role_repo = PgRoleRepo::new(pg_pool.clone());

        let session_cache = RedisSessionCache::new(redis_conn.clone());
        let perm_cache = RedisPermissionCache::new(redis_conn.clone());

        let auth_service = Arc::new(AuthService::new(AuthServiceInit {
            user_repo,
            session_repo,
            role_repo,
            session_cache,
            perm_cache,
            token: TokenConfig::from(config),
            session: SessionConfig::from(config),
        }));

        Self {
            config: Arc::new(config.to_owned()),
            auth_service,
        }
    }
}
