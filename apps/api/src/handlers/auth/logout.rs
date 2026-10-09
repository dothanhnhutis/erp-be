use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde_json::json;

use crate::{error::ApiError, state::AppState};

pub async fn logout_handler(
    State(state): State<AppState>,
    // data: AuthContext,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    // state
    //     .auth_service
    //     .logout(data.session.id, &data.session.token_hash)
    //     .await?;

    let jar = jar
        .remove(Cookie::build(state.config.access_token_name.clone()).path("/"))
        .remove(Cookie::build(state.config.refresh_token_name.clone()).path("/api/v1/auth"));
    Ok((jar, Json(json!({ "message": "Đã đăng xuất" }))))
}
