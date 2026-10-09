use application::security::tokens::Claims;
use axum::{Json, response::IntoResponse};

use crate::{error::ApiError, extractors::jwt::AuthClaims};

pub async fn me_handler(
    AuthClaims(jwt): AuthClaims<Claims>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(jwt))
}
