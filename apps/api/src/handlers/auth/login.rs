use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::{HeaderMap, request::Parts},
};
use std::time::{Duration, Instant};

#[derive(Debug)]
// an extractor that wraps another and measures how long time it takes to run
pub struct Timing<E> {
    pub extractor: E,
    pub duration: Duration,
}

// we must implement both `FromRequestParts`
impl<S, T> FromRequestParts<S> for Timing<T>
where
    S: Send + Sync,
    T: FromRequestParts<S>,
{
    type Rejection = T::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let start = Instant::now();
        let extractor = T::from_request_parts(parts, state).await?;
        let duration = start.elapsed();
        Ok(Timing {
            extractor,
            duration,
        })
    }
}

// and `FromRequest`
impl<S, T> FromRequest<S> for Timing<T>
where
    S: Send + Sync,
    T: FromRequest<S>,
{
    type Rejection = T::Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let start = Instant::now();
        let extractor = T::from_request(req, state).await?;
        let duration = start.elapsed();
        Ok(Timing {
            extractor,
            duration,
        })
    }
}

pub async fn login_handler(a: Timing<HeaderMap>, b: Timing<String>) -> &'static str {
    println!("{:#?}", a.duration);
    println!("{:#?}", a.extractor);
    println!("{:#?}", b.duration);
    println!("{:#?}", b.extractor);
    "Ok"
}
