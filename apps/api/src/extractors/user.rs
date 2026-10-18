use application::errors::AppError;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request,
};
use domain::entities::user::User;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Clone)]
pub struct Me(pub User);

impl<S> FromRequestParts<S> for Me
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Me>().cloned().ok_or_else(|| {
            ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
        })
    }
}
