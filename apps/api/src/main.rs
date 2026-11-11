mod error;
mod extractor;
mod handler;
mod helper;
mod router;
mod state;

use infrastructure::postgres::pool::create_db_pool;
use shared::config::AppConfig;
use std::{error::Error, net::SocketAddr};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{router::create_routes, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let config = AppConfig::from_env()?;

    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.log_filter))
        .with(fmt::layer().json().flatten_event(true)) // flatten giúp đẩy các field tùy biến ra ngoài layer gốc của JSON
        .init();

    let pg_pool = create_db_pool(&config.db_url).await?;

    let shared_state = AppState::new(config.clone());

    let app = create_routes().with_state(shared_state);

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
