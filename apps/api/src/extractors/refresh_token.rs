use application::security::tokens;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;

use crate::{
    error::ApiError,
    extractors::token::{cookie_token, header_token, unauthorized},
    state::AppState,
};

/// Web: httpOnly cookie (path /api/v1/auth). Mobile/desktop: header X-Refresh-Token.
const REFRESH_HEADER: &str = "x-refresh-token";

pub struct RefreshToken {
    pub session_id: uuid::Uuid,
    pub secret: String,
}

impl<S> FromRequestParts<S> for RefreshToken
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let jar = CookieJar::from_headers(&parts.headers);

        // Cookie trước, header riêng sau. KHÔNG đọc Authorization.
        let raw = cookie_token(&jar, &app_state.config.refresh_token_name)
            .or_else(|| header_token(&parts.headers, REFRESH_HEADER))
            .ok_or_else(unauthorized)?;

        let (session_id, secret) = tokens::parse_refresh(&raw).ok_or_else(unauthorized)?;

        Ok(Self {
            session_id,
            secret: secret.to_owned(),
        })
    }
}
