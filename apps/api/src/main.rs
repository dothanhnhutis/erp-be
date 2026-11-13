mod errors;
mod extractors;
mod handler;
mod helpers;
mod middlewares;
mod router;
mod state;

use shared::config::AppConfig;
use std::{error::Error, net::SocketAddr};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{router::create_routes, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;

    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.log_filter))
        .with(fmt::layer().json().flatten_event(true)) // flatten giúp đẩy các field tùy biến ra ngoài layer gốc của JSON
        .init();

    let shared_state = AppState::new(&config).await;

    let app = create_routes(shared_state.clone()).with_state(shared_state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("server stated on {addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
