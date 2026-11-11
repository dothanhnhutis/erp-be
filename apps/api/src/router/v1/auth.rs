use axum::{Router, routing::post};

use crate::handler::auth::login::login_handler;

pub fn router_v1() -> Router {
    Router::new().route("/auth/login", post(login_handler))
}
