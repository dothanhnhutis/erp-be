use crate::error::ApiError;
use application::errors::AppError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use domain::entities::{session::Session, user::User};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub infor: User,
    pub session: Session,
    pub permission_codes: Vec<String>,
}

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<User>().cloned().ok_or_else(|| {
            ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
        })
    }
}
