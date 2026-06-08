// ── External crates ──
use axum::{Json, extract::State, http::StatusCode};
use dartdetect::dart_game::{game::GameSession, x01::X01};

// ── Internal modules ──
use super::helpers::{emit_event, game_state_from_session};
use crate::{
    error::ApiError,
    runtime::{AppState, ServerEvent},
};

/// Request body for the `POST /api/game/start` endpoint.
#[derive(serde::Deserialize, Debug)]
pub(crate) struct StartGameRequest {
    #[serde(rename = "playersCount")]
    players_count: u32,
    #[serde(rename = "startingScore")]
    starting_score: u32,
}

/// Starts a new X01 game session with the given player count and starting score.
///
/// The created session is stored in the runtime and its initial state is broadcast to all WS clients.
pub(crate) async fn start_game_handler(
    State(state): State<AppState>,
    Json(params): Json<StartGameRequest>,
) -> Result<StatusCode, ApiError> {
    println!(
        "Game start requested: {} players, starting score = {}",
        params.players_count, params.starting_score
    );

    // Create new game session with generated player names
    let session = GameSession::new(
        X01::new(params.starting_score, false),
        (1..=params.players_count)
            .map(|i| format!("Player {}", i))
            .collect(),
    );

    // Build game state payload and broadcast to all WS clients
    let game_state = game_state_from_session(&session);
    emit_event(&state.runtime, ServerEvent::GameState(game_state.clone()));

    // Update active game in runtime state
    *state.runtime.active_game.write().await = Some(session);

    Ok(StatusCode::OK)
}
