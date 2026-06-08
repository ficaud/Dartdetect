use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// Mainly internal handlers list
use crate::{
    handlers::{
        latest_score_handler, start_game_handler, status_handler, version_handler, ws_handler,
    },
    runtime::AppState,
};

/// Defines the main router for the application, mapping HTTP endpoints to their respective handlers,
/// and applying necessary middleware layers (CORS, tracing).
///
/// # Arguments
/// - `state`: The shared application state containing the simulation runtime, passed to handlers via Axum's state management.
///
/// # Returns
/// - `Router`: The configured Axum router with all routes and middleware applied, ready to be served by the application.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/version", get(version_handler))
        .route("/api/status", get(status_handler))
        .route("/api/game/start", post(start_game_handler))
        .route("/api/latest", get(latest_score_handler))
        .route("/ws/impacts", get(ws_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
