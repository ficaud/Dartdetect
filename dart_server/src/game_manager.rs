use crate::handlers::game_state_from_session;
use crate::runtime::{GameStatePayload, ScorePayload, ServerEvent, SimulationRuntime};
use dartdetect::{
    dart_core::Point,
    dart_game::{self},
    dart_interpretor::score::Score,
};

/// Applies a dart throw at coordinates (x, y) to the active game session.
/// Returns `None` if no game is active, or `Some((game_state, (winner_index, winner_name)))` otherwise.
pub(crate) async fn apply_shot(
    runtime: &SimulationRuntime,
    x: f64,
    y: f64,
) -> Option<(GameStatePayload, Option<(usize, String)>)> {
    let has_game = runtime.active_game.read().await.is_some();
    if !has_game {
        return None;
    }

    // Derive ShotResult (sector + multiplier) from the impact coordinates
    let shot_result = {
        let mut score = Score::new(Point::new(x, y));
        score.get_shortresult()
    };

    // Apply the shot to the active game session
    let mut game = runtime.active_game.write().await;
    let session = game.as_mut()?;

    let outcome = session.apply_shot(shot_result);
    let game_state = game_state_from_session(session);

    let game_over = outcome.game_over.then(|| {
        let winner = match session.phase {
            dart_game::game::GamePhase::Finished(i) => i,
            _ => session.current_player,
        };
        (winner, session.players[winner].name.clone())
    });

    Some((game_state, game_over))
}

/// Handles the common post-impact flow: save score, apply shot if game active, return events to broadcast.
pub(crate) async fn process_impact(
    runtime: &SimulationRuntime,
    impact_x: f64,
    impact_y: f64,
    score: u32,
) -> Vec<ServerEvent> {
    let score_payload = ScorePayload {
        impact_x,
        impact_y,
        score,
    };

    // Save latest score
    {
        let mut latest = runtime.latest_score.write().await;
        *latest = Some(score_payload.clone());
    }

    let mut events = Vec::new();

    // Check if a game is active
    if let Some((game_state, game_over)) = apply_shot(runtime, impact_x, impact_y).await {
        // Always emit Score so the frontend updates the impact dot
        events.push(ServerEvent::Score(score_payload));
        events.push(ServerEvent::GameState(game_state));

        if let Some((idx, name)) = game_over {
            events.push(ServerEvent::GameOver {
                winner_index: idx,
                winner_name: name,
            });
        }
    } else {
        events.push(ServerEvent::Score(score_payload));
    }

    events
}
