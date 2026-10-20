use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde_json::json;

use crate::{error::ApiError, helpers::cookie::clear_auth_cookies, state::AppState};

pub async fn logout_handler(
    State(state): State<AppState>,
    // data: AuthContext,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    // state
    //     .auth_service
    //     .logout(data.session.id, &data.session.token_hash)
    //     .await?;

    let jar = clear_auth_cookies(jar, &state);

    Ok((jar, Json(json!({ "message": "Đã đăng xuất" }))))
}
