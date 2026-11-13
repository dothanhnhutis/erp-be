use axum::{Router, extract::FromRef, routing::post};

use crate::{
    handler::auth::{login::login_handler, logout::logout_handler},
    state::AppState,
};

pub fn create_public_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/login", post(login_handler))
}

pub fn create_private_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/logout", post(logout_handler))
}
