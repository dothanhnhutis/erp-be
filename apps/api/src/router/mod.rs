pub mod v1;

use crate::router::v1::auth::router_v1;
use axum::{Router, routing::get};

async fn health_route() -> &'static str {
    "Ok"
}

fn router_version() -> Router {
    Router::new().nest("/v1", router_v1())
}

pub fn create_routes() -> Router {
    Router::new()
        .route("/health", get(health_route))
        .nest("/api", router_version())
}
