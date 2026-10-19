use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::{error::ApiError, extractors::user::Me};

pub async fn me_handler(
    Me(user): Me,
    // UserSession(session): UserSession,
    // Permissions(perms): Permissions,
    // _: Require<UserRead>,
) -> Result<impl IntoResponse, ApiError> {
    Ok((StatusCode::OK, Json(user)))
}
