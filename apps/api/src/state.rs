use std::sync::Arc;

use axum::extract::FromRef;
use shared::config::AppConfig;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}
