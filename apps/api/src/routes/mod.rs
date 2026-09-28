mod v1;
use axum::{Router, routing::get};

async fn health_check_handler() -> &'static str {
    "OK"
}

pub fn create_router() -> Router {
    Router::new()
        .route("/health-check", get(health_check_handler))
        .nest("/v1", v1::create_routes())
}
