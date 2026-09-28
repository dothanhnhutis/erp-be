use axum::{Router, routing::get};

use crate::handlers::user::me_handler;

pub fn create_routes() -> Router {
    Router::new().route("/me", get(me_handler))
}
