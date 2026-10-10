use axum::{Router, extract::FromRef, routing::post};

use crate::{handlers::auth, state::AppState};

pub fn create_public_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/login", post(auth::login_handler))
}

pub fn create_private_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/logout", post(auth::logout_handler))
}
