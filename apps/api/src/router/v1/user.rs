use axum::{Router, extract::FromRef, routing::get};

use crate::{handler::user::me::me_handler, state::AppState};

pub fn create_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/me", get(me_handler))
}
