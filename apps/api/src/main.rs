mod http;
mod logger;

use axum::{Router, routing::get};
use http::RouterExt;
use shared::config::AppConfig;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load env: .env không bắt buộc (prod có thể set biến thật); file sai cú pháp vẫn báo lỗi
    if let Err(e) = dotenvy::dotenv() {
        if !e.not_found() {
            return Err(e.into());
        }
    }
    // config
    let config: AppConfig = AppConfig::from_env()?;

    // init log
    let _guard = logger::init(&config)?;

    let app = Router::new()
        .route("/health-check", get(|| async { "OK" }))
        .with_http_tracing();

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
