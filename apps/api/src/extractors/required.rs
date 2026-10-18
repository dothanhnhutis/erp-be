use std::marker::PhantomData;

use application::errors::AppError;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request,
};

use crate::{
    error::ApiError,
    extractors::permission::{PermissionCode, Permissions},
    state::AppState,
};

/// Chặn request nếu user không có quyền P. Dùng: `_: Require<UserRead>`.
pub struct Require<P>(PhantomData<fn() -> P>);

impl<S, P> FromRequestParts<S> for Require<P>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
    P: PermissionCode,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        // Đọc trực tiếp từ extensions, không clone cả Vec như Permissions extractor.
        let Permissions(codes) = parts.extensions.get::<Permissions>().ok_or_else(|| {
            ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
        })?;

        if codes.iter().any(|c| c == P::CODE) {
            Ok(Self(PhantomData))
        } else {
            Err(ApiError::Domain(AppError::Forbidden(format!(
                "Bạn không có quyền: {}",
                P::CODE
            ))))
        }
    }
}
