mod v1;
use axum::{Router, extract::FromRef, routing::get};

use crate::state::AppState;

async fn health_check_handler() -> &'static str {
    "OK"
}

pub fn create_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new()
        .route("/health-check", get(health_check_handler))
        .nest("/v1", v1::create_routes())
}
