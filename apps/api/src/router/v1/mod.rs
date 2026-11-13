mod auth;
mod user;

use crate::{middlewares::auth_middleware, state::AppState};
use axum::{Router, extract::FromRef, middleware::from_fn_with_state};

pub fn router_v1<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let public_route = Router::new().nest("/auth", auth::create_public_routes());

    let private_route = Router::new()
        .nest("/users", user::create_routes())
        .nest("/auth", auth::create_private_routes())
        .route_layer(from_fn_with_state(state, auth_middleware::require_auth));

    public_route.merge(private_route)
}
