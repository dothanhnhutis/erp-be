use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde_json::json;

use crate::{
    error::ApiError, extractors::session::UserSession, helpers::cookie::clear_auth_cookies,
    state::AppState,
};

pub async fn logout_handler(
    State(state): State<AppState>,
    UserSession(session): UserSession,
    jar: CookieJar,
) -> Response {
    let jar = clear_auth_cookies(jar, &state);
    match state.auth_service.logout(&session).await {
        Ok(()) => (jar, Json(json!({ "message": "Đã đăng xuất" }))).into_response(),
        Err(e) => (jar, ApiError::from(e)).into_response(),
    }
}
