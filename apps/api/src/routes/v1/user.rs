use std::sync::Arc;

use axum::{Router, extract::FromRef, routing::get};
use shared::config::AppConfig;

use crate::{handlers::user, state::AppState};

pub fn create_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/me", get(user::me_handler))
}
