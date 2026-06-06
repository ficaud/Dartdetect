use dartdectec::dart_game::{game::GameSession, x01::X01, ScorePayload};
use serde::Serialize;
use std::sync::{
    Arc,
    atomic::AtomicBool,
};
use tokio::sync::{RwLock, broadcast};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) runtime: Arc<SimulationRuntime>,
}

pub(crate) struct SimulationRuntime {
    pub(crate) running: AtomicBool,
    pub(crate) latest_score: RwLock<Option<ScorePayload>>,
    pub(crate) active_game: RwLock<Option<GameSession<X01>>>,
    pub(crate) events_tx: broadcast::Sender<ServerEvent>,
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StatusPayload {
    pub(crate) running: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GameStatePayload {
    pub players: Vec<PlayerState>,
    pub current_player: usize,
    pub current_dart: u32,
    pub phase: String,
    pub winner: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PlayerState {
    pub name: String,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub(crate) enum ServerEvent {
    Score(ScorePayload),
    Status(StatusPayload),
    GameState(GameStatePayload),
    GameOver { winner_index: usize, winner_name: String },
    Error(String),
}
