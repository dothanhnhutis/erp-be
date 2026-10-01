use axum::{
    Json,
    extract::{FromRequest, rejection::JsonRejection},
};
use serde::de::DeserializeOwned;

use crate::error::ApiError;

#[derive(Debug, Clone, Copy)]
pub struct ValidatedBodyJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedBodyJson<T>
where
    T: DeserializeOwned + validator::Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedBodyJson(value))
    }
}
