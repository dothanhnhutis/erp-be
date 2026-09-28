mod auth;
mod user;
use axum::Router;

pub fn create_routes() -> Router {
    Router::new()
        .nest("/auth", auth::create_routes())
        .nest("/users", user::create_routes())
}
