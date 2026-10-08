mod error;
mod extractors;
mod handlers;
mod http;
mod logger;
mod routes;
mod state;

use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method, header},
};
use core::net::SocketAddr;
use http::RouterExt;
use routes::create_router;
use shared::config::AppConfig;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::init_state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load env: .env không bắt buộc (prod có thể set biến thật); file sai cú pháp vẫn báo lỗi
    if let Err(e) = dotenvy::dotenv()
        && !e.not_found()
    {
        tracing::error!("{e}");
    }

    // config
    let config: AppConfig = AppConfig::from_env()?;

    // init log
    // let _guard = logger::init(&config)?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.log_filter))
        .with(fmt::layer().json().flatten_event(true)) // flatten giúp đẩy các field tùy biến ra ngoài layer gốc của JSON
        .init();

    // cors
    let origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty()) // Bỏ qua nếu có phần tử rỗng
        .filter_map(|s| s.parse::<HeaderValue>().ok()) // Parse an toàn, bỏ qua phần tử lỗi
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            HeaderName::from_static("x-requested-with"),
        ]);

    // state
    let shared_state = init_state(config.clone()).await;

    let app = Router::new()
        .nest("/api", create_router())
        .layer(cors)
        .with_http_tracing()
        .with_state(shared_state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
