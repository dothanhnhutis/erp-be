mod logger;

use axum::{Router, routing::get};
use shared::config::AppConfig;
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //log
    let _guard = logger::init_tracing()?;

    // env
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".into());
    dotenvy::from_filename(format!(".env.{app_env}"))?;
    let config = AppConfig::from_env()?;

    // build our application with a single route
    let app = Router::new()
        .route("/health-check", get(|| async { "OK" }))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(true),
                )
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        );
    // .layer(
    //     TraceLayer::new_for_http().make_span_with(
    //         DefaultMakeSpan::new()
    //             .level(Level::INFO)
    //             .include_headers(true),
    //     ),
    // )
    // .layer(SetSensitiveRequestHeadersLayer::new([
    //     AUTHORIZATION,
    //     COOKIE,
    // ]));

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
