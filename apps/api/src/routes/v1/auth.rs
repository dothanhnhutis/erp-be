use axum::{Router, routing::post};

use crate::handlers::auth;

pub fn create_routes() -> Router {
    Router::new().route("/login", post(auth::login_handler))
}
