use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::{error::ApiError, extractors::jwt::Authentication};

pub async fn me_handler(
    Authentication(jwt): Authentication,
) -> Result<impl IntoResponse, ApiError> {
    Ok((StatusCode::OK, Json(jwt)))
}
