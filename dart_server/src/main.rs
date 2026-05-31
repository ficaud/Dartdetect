mod error;
mod handlers;
mod ws;
mod routes;
mod runtime;

use std::{
    env,
    net::SocketAddr,
    sync::Arc,
};
use tokio::net::TcpListener;
use tracing::info;
use crate::runtime::{AppState, SimulationRuntime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dart_server=info,tower_http=info".into()),
        )
        .init();

    let host = env::var("DART_SERVER_HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let port = env::var("DART_SERVER_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let address: SocketAddr = format!("{}:{}", host, port).parse()?;

    let state = AppState {
        runtime: Arc::new(SimulationRuntime::new()),
    };

    let app = routes::router(state);

    info!("dart_server listening on http://{}", address);

    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
