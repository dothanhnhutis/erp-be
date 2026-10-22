use axum::{Router, extract::FromRef, routing::put};

use crate::{handlers::role, state::AppState};

pub fn create_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route(
        "/{id}/permissions",
        put(role::update_role_permissions_handler),
    )
}
