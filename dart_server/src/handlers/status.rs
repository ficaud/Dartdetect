// ── External crates ──
use axum::{extract::State, http::StatusCode, Json};

// ── Internal modules ──
use crate::{
    error::ApiError,
    runtime::{AppState, ScorePayload, StatusPayload},
};
use super::helpers::status_from_runtime;

/// Returns the current simulation status (running/stopped) as JSON.
pub(crate) async fn status_handler(State(state): State<AppState>) -> Json<StatusPayload> {
    Json(status_from_runtime(&state.runtime))
}

/// Returns the most recent score (impact coordinates + points) as JSON, or a 404 if none exists.
pub(crate) async fn latest_score_handler(
    State(state): State<AppState>,
) -> Result<Json<ScorePayload>, ApiError> {
    let latest = state.runtime.latest_score.read().await;
    if let Some(payload) = latest.clone() {
        return Ok(Json(payload));
    }

    Err(ApiError::new(
        StatusCode::NOT_FOUND,
        "no score has been produced yet",
    ))
}

/// Returns the server version string.
pub(crate) async fn version_handler() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
