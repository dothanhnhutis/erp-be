mod error;
mod extractors;
mod handlers;
mod http;
mod logger;
mod routes;
mod state;

use axum::Router;
use http::RouterExt;
use shared::config::AppConfig;
use tokio::net::TcpListener;

use routes::create_router;
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

    // state
    let shared_state = init_state(&config).await;

    let app = Router::new()
        .nest("/api", create_router())
        .with_http_tracing()
        .with_state(shared_state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
