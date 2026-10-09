use axum::{
    extract::{FromRef, FromRequestParts},
    http::request,
};

use crate::{error::ApiError, state::AppState};

pub struct CurrentUser {}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let (session, user, permission_codes) = app_state.auth_service.authenticate(&token).await?;

        Ok(CurrentUser {})
    }
}
