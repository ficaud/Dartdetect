
use axum::{
    Router, routing::{get, post}
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// Mainly internal handlers list
use crate::{
    handlers::{
        latest_score_handler,
        status_handler,
        ws_handler,
        version_handler,
        start_game_handler,
    },
    runtime::AppState,
};

// Defines all the HTTP routes of the server, and their corresponding handler functions.
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