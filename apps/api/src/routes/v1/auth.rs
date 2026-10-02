use axum::{Router, extract::FromRef, routing::post};

use crate::{handlers::auth, state::AppState};

pub fn create_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/login", post(auth::login_handler))
}
