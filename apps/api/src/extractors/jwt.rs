use crate::error::ApiError;
use crate::state::AppState;
use application::errors::AppError;
use application::security::tokens::{self, Claims};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};
use axum_extra::extract::CookieJar;

// JWT
//  ↓
// Authentication
//  ↓
// CurrentUser
//  ↓
// Authorization
//  ↓
// CHEMICAL_CREATE
//  ↓
// Handler

pub struct Authentication(pub Claims);

impl<S> FromRequestParts<S> for Authentication
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
    // Arc<AppConfig>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        // Arc<AppConfig>
        // let config = Arc::<AppConfig>::from_ref(state);
        let jar = CookieJar::from_headers(&parts.headers);
        let token = extract_token(&parts.headers, &jar, &app_state.config.access_token_name)
            .ok_or_else(|| {
                ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
            })?;

        let claims = tokens::verify_access(&app_state.config.jwt_dec, &token)?;

        Ok(Authentication(claims))
    }
}

fn extract_token(headers: &HeaderMap, jar: &CookieJar, key: &str) -> Option<String> {
    if let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        && let Some(token) = value.strip_prefix("Bearer ")
    {
        let token = token.trim();
        if !token.is_empty() {
            return Some(token.to_owned());
        }
    }
    jar.get(key).map(|c| c.value().to_owned())
}
