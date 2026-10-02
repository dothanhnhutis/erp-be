mod auth;
mod user;
use axum::{Router, extract::FromRef};

use crate::state::AppState;

pub fn create_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new()
        .nest("/auth", auth::create_routes())
        .nest("/users", user::create_routes())
}
