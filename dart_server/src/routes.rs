
use axum::{
    Router, routing::{get, post}
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    handlers::{
        health_handler,
        latest_score_handler,
        status_handler,
        ws_handler,
        version_handler,
        start_game_handler,
    },
    runtime::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/version", get(version_handler))
        .route("/api/status", get(status_handler))
        .route("/api/game/start", post(start_game_handler))
        .route("/api/latest", get(latest_score_handler))
        .route("/ws/impacts", get(ws_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}