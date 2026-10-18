use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::{
    error::ApiError,
    extractors::{permission::Permissions, required::Require, session::UserSession, user::Me},
    permissions::UserRead,
};

pub async fn me_handler(
    Me(user): Me,
    UserSession(session): UserSession,
    Permissions(codes): Permissions,
    _: Require<UserRead>,
) -> Result<impl IntoResponse, ApiError> {
    Ok((StatusCode::OK, Json(user)))
}
