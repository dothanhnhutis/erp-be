use application::errors::AppError;
use axum::http::{HeaderMap, header};
use axum_extra::extract::CookieJar;

use crate::error::ApiError;

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

pub fn cookie_token(jar: &CookieJar, name: &str) -> Option<String> {
    jar.get(name).map(|c| c.value().to_owned())
}

pub fn header_token(headers: &HeaderMap, name: &str) -> Option<String> {
    let token = headers.get(name)?.to_str().ok()?.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

pub fn unauthorized() -> ApiError {
    ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
}
