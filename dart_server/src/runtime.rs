use dartdectec::dart_game::{game::GameSession, x01::X01};
use serde::Serialize;
use std::sync::{
    Arc,
    atomic::AtomicBool,
};
use tokio::sync::{RwLock, broadcast};

/// Injected into every Axum route handler — wraps the shared runtime state in an Arc.
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) runtime: Arc<SimulationRuntime>,
}

/// Central shared state of the server: simulation flag, last score, active game, and event bus.
pub(crate) struct SimulationRuntime {
    /// Whether the automatic demo simulation is running.
    pub(crate) running: AtomicBool,
    /// The most recent score payload, sent to newly connecting WS clients.
    pub(crate) latest_score: RwLock<Option<ScorePayload>>,
    /// The current game session (None = no game started yet).
    pub(crate) active_game: RwLock<Option<GameSession<X01>>>,
    /// Broadcast channel that distributes events to all connected WebSocket clients.
    pub(crate) events_tx: broadcast::Sender<ServerEvent>,
}


/// Result of a dart throw: impact coordinates and the points scored.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ScorePayload {
    pub impact_x: f64,
    pub impact_y: f64,
    pub score: u32,
}

/// Events broadcast through the event bus to all WS clients.
/// Serialized as JSON with a `type` discriminator (e.g. `{ "type": "score", "payload": ... }`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub(crate) enum ServerEvent {
    /// A dart throw result (coordinates + score).
    Score(ScorePayload),
    /// Simulation status (running / stopped).
    Status(StatusPayload),
    /// Current state of the active game (players, scores, turn).
    GameState(GameStatePayload),
    /// A player has won the game.
    GameOver { winner_index: usize, winner_name: String },
    /// An error occurred.
    Error(String),
}

impl SimulationRuntime {
    
    pub(crate) fn new() -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            running: AtomicBool::new(false),
            latest_score: RwLock::new(None),
            active_game: RwLock::new(None),
            events_tx,
        }
    }
}

/// Whether the automatic demo simulation is running or stopped.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct StatusPayload {
    pub(crate) running: bool,
}

/// Snapshot of the active game sent after each throw: player states, current turn, and winner.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GameStatePayload {
    pub players: Vec<PlayerState>,
    pub current_player: usize,
    pub current_dart: u32,
    pub phase: String,
    pub winner: Option<usize>,
}

/// A single player's name and remaining score within the game.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PlayerState {
    pub name: String,
    pub score: u32,
}