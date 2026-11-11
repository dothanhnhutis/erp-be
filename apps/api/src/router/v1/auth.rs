use axum::{Router, extract::FromRef, routing::post};

use crate::{handler::auth::login::login_handler, state::AppState};

pub fn router_v1<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/auth/login", post(login_handler))
}
