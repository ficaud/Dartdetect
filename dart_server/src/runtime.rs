use dartdectec::dart_game::ScorePayload;
use serde::Serialize;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64},
};
use tokio::sync::{RwLock, broadcast};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) runtime: Arc<SimulationRuntime>,
}

pub(crate) struct SimulationRuntime {
    pub(crate) running: AtomicBool,
    pub(crate) interval_ms: AtomicU64,
    pub(crate) latest_score: RwLock<Option<ScorePayload>>,
    pub(crate) events_tx: broadcast::Sender<ServerEvent>,
}

impl SimulationRuntime {
    pub(crate) fn new() -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            running: AtomicBool::new(false),
            interval_ms: AtomicU64::new(1000),
            latest_score: RwLock::new(None),
            events_tx,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StatusPayload {
    pub(crate) running: bool,
    pub(crate) interval_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub(crate) enum ServerEvent {
    Score(ScorePayload),
    Status(StatusPayload),
    Error(String),
}
