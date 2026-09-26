use axum::{
    Router,
    extract::Request,
    http::header::{AUTHORIZATION, COOKIE},
};
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;

pub trait RouterExt {
    /// Gọi SAU khi đã khai báo hết route
    fn with_http_tracing(self) -> Self;
}

impl<S: Clone + Send + Sync + 'static> RouterExt for Router<S> {
    fn with_http_tracing(self) -> Self {
        // ServiceBuilder: layer khai báo TRƯỚC là layer bọc NGOÀI, chạy trước
        let middleware = ServiceBuilder::new()
            .layer(SetSensitiveRequestHeadersLayer::new([
                AUTHORIZATION,
                COOKIE,
            ]))
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|req: &Request| {
                        let request_id = req
                            .headers()
                            .get("x-request-id")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("-");
                        tracing::info_span!(
                            "request",
                            method = %req.method(),
                            uri = %req.uri(),
                            request_id,
                        )
                    })
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Millis),
                    ),
            )
            .layer(PropagateRequestIdLayer::x_request_id());

        self.layer(middleware)
    }
}
