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
    P: PermissionRule,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let perms = parts.extensions.get::<Permissions>().ok_or_else(|| {
            ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
        })?;

        if P::allows(perms) {
            Ok(Self(PhantomData))
        } else {
            Err(ApiError::Domain(AppError::Forbidden(format!(
                "Bạn không có quyền: {}",
                P::describe()
            ))))
        }
    }
}

pub trait PermissionRule {
    fn allows(perms: &Permissions) -> bool;
    fn describe() -> String;
}

// Mọi PermissionCode đều là một rule đơn
impl<P: PermissionCode> PermissionRule for P {
    fn allows(perms: &Permissions) -> bool {
        perms.has(P::CODE)
    }
    fn describe() -> String {
        P::CODE.to_string()
    }
}

pub struct AnyOf<A, B>(PhantomData<fn() -> (A, B)>);
impl<A: PermissionRule, B: PermissionRule> PermissionRule for AnyOf<A, B> {
    fn allows(c: &Permissions) -> bool {
        A::allows(c) || B::allows(c)
    }
    fn describe() -> String {
        format!("{} | {}", A::describe(), B::describe())
    }
}

pub struct AllOf<A, B>(PhantomData<fn() -> (A, B)>);
impl<A: PermissionRule, B: PermissionRule> PermissionRule for AllOf<A, B> {
    fn allows(c: &Permissions) -> bool {
        A::allows(c) && B::allows(c)
    }
    fn describe() -> String {
        format!("{} & {}", A::describe(), B::describe())
    }
}
