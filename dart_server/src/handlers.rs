// ── Standard library ──
use std::sync::atomic::Ordering;

// ── External crates ──
use axum::{
    extract::{State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use dartdetect::dart_game::{game::GameSession, x01::X01};

// ── Internal modules ──
use crate::{
    error::ApiError,
    runtime::{
        AppState, GameStatePayload, PlayerState, ScorePayload, ServerEvent,
        SimulationRuntime, StatusPayload,
    },
    ws::ws_connection,
};

// Structure that store the request body for the "start game" endpoint.
#[derive(serde::Deserialize, Debug)]
pub(crate) struct StartGameRequest {
    #[serde(rename = "playersCount")]
    players_count: u32,
    #[serde(rename = "startingScore")]
    starting_score: u32,
}

// Handler for the /api/status endpoint: returns the current simulation status.
pub(crate) fn status_from_runtime(runtime: &SimulationRuntime) -> StatusPayload {
    StatusPayload {
        running: runtime.running.load(Ordering::Relaxed)
    }
}

// Handler helper that takes runtime and a ServerEvent, and broadcasts the event to all WS clients.
pub(crate) fn emit_event(runtime: &SimulationRuntime, event: ServerEvent) {
    let _ = runtime.events_tx.send(event);
}

// Helper that converts a GameSession into a GameStatePayload for broadcasting to WS clients.
pub(crate) fn game_state_from_session(session: &GameSession<X01>) -> GameStatePayload {
    GameStatePayload {
        players: session
            .players
            .iter()
            .map(|p| PlayerState {
                name: p.name.clone(),
                score: p.data,
            })
            .collect(),
        current_player: session.current_player,
        current_dart: session.current_dart,
        phase: match session.phase {
            dartdetect::dart_game::game::GamePhase::Playing => "playing".into(),
            dartdetect::dart_game::game::GamePhase::Finished(w) => format!("finished:{}", w),
        },
        winner: match session.phase {
            dartdetect::dart_game::game::GamePhase::Finished(w) => Some(w),
            _ => None,
        },
    }
}

// Handler that reports the server stateus to the /api/status endpoint.
pub(crate) async fn status_handler(State(state): State<AppState>) -> Json<StatusPayload> {
    Json(status_from_runtime(&state.runtime))
}

// Handler for the /api/latest endpoint: returns the most recent score as JSON that includes the impact coordinates and the points scored.
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

// Handler that iniitiates a WebSocket connection at the /ws/impacts endpoint,
// and starts sending real-time updates about dart throws, game state, and simulation status to connected clients.
pub(crate) async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connection(socket, state))
}

// Handler for the /api/game/start endpoint: starts a new game session with the specified parameters (number of players, starting score).
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
        (1..=params.players_count).map(|i| format!("Player {}", i)).collect(),
    );

    // Build game state payload and broadcast to all WS clients
    let game_state = game_state_from_session(&session);
    emit_event(&state.runtime, ServerEvent::GameState(game_state.clone()));

    // Update active game in runtime state
    *state.runtime.active_game.write().await = Some(session);

    Ok(StatusCode::OK)
}

// Handler for the /version endpoint: returns the current server version from Cargo.toml.
pub(crate) async fn version_handler() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
