// ── Standard library ──
use std::sync::atomic::Ordering;

// ── Internal modules ──
use crate::runtime::{
    GameStatePayload, PlayerState, ServerEvent, SimulationRuntime, StatusPayload,
};
use dartdetect::dart_game::{game::GameSession, x01::X01};

/// Reads the `running` flag from the runtime and returns it as a [`StatusPayload`].
pub(crate) fn status_from_runtime(runtime: &SimulationRuntime) -> StatusPayload {
    StatusPayload {
        running: runtime.running.load(Ordering::Relaxed),
    }
}

/// Sends a [`ServerEvent`] through the runtime's broadcast channel.
///
/// Errors (e.g. no active subscribers) are silently ignored.
pub(crate) fn emit_event(runtime: &SimulationRuntime, event: ServerEvent) {
    let _ = runtime.events_tx.send(event);
}

/// Converts a [`GameSession`] into a [`GameStatePayload`] suitable for client consumption.
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
