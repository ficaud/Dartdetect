use axum::{
    extract::{
        State,
        ws::WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use dartdectec::dart_game::{ScorePayload, game::GameSession, x01::X01};
use serde::Serialize;
use std::{
    fmt::Debug, sync::{
        atomic::Ordering,
    }
};

use crate::{
    error::ApiError,
    runtime::{AppState, ServerEvent, SimulationRuntime, StatusPayload},
};
use crate::ws::{ws_connection};

#[derive(serde::Deserialize, Debug)]
pub(crate) struct StartGameRequest {
    #[serde(rename = "playersCount")]
    players_count: u32,
    #[serde(rename = "startingScore")]
    starting_score: u32,
}


#[derive(Debug, Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
}

pub(crate) fn status_from_runtime(runtime: &SimulationRuntime) -> StatusPayload {
    StatusPayload {
        running: runtime.running.load(Ordering::Relaxed)
    }
}
pub(crate) fn emit_event(runtime: &SimulationRuntime, event: ServerEvent) {
    let _ = runtime.events_tx.send(event);
}

pub(crate) async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub(crate) async fn status_handler(State(state): State<AppState>) -> Json<StatusPayload> {
    Json(status_from_runtime(&state.runtime))
}

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

pub(crate) async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connection(socket, state))
}

pub(crate) async fn start_game_handler(
    State(state): State<AppState>,
    Json(params): Json<StartGameRequest>,
) -> Result<StatusCode, ApiError> {
    println!(
        "Game start requested: {} players, starting score = {}",
        params.players_count, params.starting_score
    );

    let session = GameSession::new(
        X01::new(params.starting_score, false),
        (1..=params.players_count).map(|i| format!("Player {}", i)).collect(),
    );

    *state.runtime.active_game.write().await = Some(session);

    Ok(StatusCode::OK)
}

pub(crate) async fn version_handler() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
