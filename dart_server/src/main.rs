mod connection;
mod error;
mod game_manager;
mod handlers;
mod routes;
mod runtime;

use crate::runtime::{AppState, SimulationRuntime};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with environment variable filter (defaulting to info level for our crate and tower_http).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dart_server=info,tower_http=info".into()),
        )
        .init();

    // Read server host and port from environment variables, with defaults if not set.
    let host = env::var("DART_SERVER_HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    // Parse the port number, defaulting to 8080 if the environment variable is not set or invalid.
    let port = env::var("DART_SERVER_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    // Combine host and port into a SocketAddr for the server to bind to.
    let address: SocketAddr = format!("{}:{}", host, port).parse()?;

    // Initialize the shared application state, including the simulation runtime, and create the Axum router with this state.
    let state = AppState {
        runtime: Arc::new(SimulationRuntime::new()),
    };

    // Build the Axum application router with the defined routes and shared state.
    let app = routes::router(state);

    info!("dart_server listening on http://{}", address);

    // Bind a TCP listener to the specified address.
    let listener = TcpListener::bind(address).await?;

    // Start the Axum server with the TCP listener and the application router, and await its completion.
    axum::serve(listener, app).await?;

    Ok(())
}
