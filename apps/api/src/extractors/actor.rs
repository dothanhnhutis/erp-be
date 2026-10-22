use application::{errors::AppError, security::authorization::Actor};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request,
};

use crate::{
    error::ApiError,
    extractors::{permission::Permissions, user::Me},
    state::AppState,
};

pub struct CurrentActor(pub Actor);

impl<S> FromRequestParts<S> for CurrentActor
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let unauthorized =
            || ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()));
        let Me(user) = parts
            .extensions
            .get::<Me>()
            .cloned()
            .ok_or_else(unauthorized)?;
        let Permissions(perms) = parts
            .extensions
            .get::<Permissions>()
            .cloned()
            .ok_or_else(unauthorized)?;
        Ok(Self(Actor {
            user_id: user.id,
            permissions: perms,
        }))
    }
}
