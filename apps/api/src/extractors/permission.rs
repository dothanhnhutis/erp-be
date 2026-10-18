use application::errors::AppError;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request,
};

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Clone)]
pub struct Permissions(pub Vec<String>);

impl<S> FromRequestParts<S> for Permissions
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Permissions>()
            .cloned()
            .ok_or_else(|| {
                ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
            })
    }
}

use std::marker::PhantomData;

/// Mỗi permission là một kiểu, mã quyền nằm ở associated const.
pub trait PermissionCode {
    const CODE: &'static str;
}

/// Khai báo nhanh: permission!(UserRead, "user.read");
#[macro_export]
macro_rules! permission {
    ($name:ident, $code:literal) => {
        pub struct $name;
        impl $crate::extractors::permission::PermissionCode for $name {
            const CODE: &'static str = $code;
        }
    };
}
