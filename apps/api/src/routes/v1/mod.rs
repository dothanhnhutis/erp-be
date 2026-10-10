mod auth;
mod user;
use axum::{Router, extract::FromRef};

use crate::state::AppState;

pub fn create_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let public_route = Router::new().nest("/auth", auth::create_public_routes());

    let private_route = Router::new()
        .nest("/users", user::create_routes())
        .nest("/auth", auth::create_private_routes());

    public_route.merge(private_route)
}
