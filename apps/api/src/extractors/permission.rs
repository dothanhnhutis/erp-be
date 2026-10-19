use std::collections::HashSet;

use application::errors::AppError;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request,
};

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Clone)]
pub struct Permissions(pub HashSet<String>);

impl Permissions {
    pub fn has(&self, code: &str) -> bool {
        self.0.contains(code)
    }
}

impl From<Vec<String>> for Permissions {
    fn from(codes: Vec<String>) -> Self {
        Self(codes.into_iter().collect())
    }
}

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
