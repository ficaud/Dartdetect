use crate::runtime::{AppState, ServerEvent, GameStatePayload, PlayerState};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::{sync::Arc};
use axum::extract::ws::{Message, WebSocket};

use crate::handlers::status_from_runtime;
use crate::handlers::emit_event;
use dartdectec::dart_core::Point;
use dartdectec::dart_game::{self, game::GameSession, x01::X01};
use dartdectec::dart_interpretor::score::Score;

fn game_state_from_session(session: &GameSession<X01>) -> GameStatePayload {
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
            dartdectec::dart_game::game::GamePhase::Playing => "playing".into(),
            dartdectec::dart_game::game::GamePhase::Finished(w) => format!("finished:{}", w),
        },
        winner: match session.phase {
            dartdectec::dart_game::game::GamePhase::Finished(w) => Some(w),
            _ => None,
        },
    }
}
#[derive(serde::Deserialize, Debug)]
struct ClientCommand {
    x_pos: Option<f64>,
    y_pos: Option<f64>,
}

pub(crate) async fn ws_connection(socket: WebSocket, state: AppState) {
    let runtime = Arc::clone(&state.runtime);
    let mut rx = runtime.events_tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    let status_event = ServerEvent::Status(status_from_runtime(&runtime));
    if send_event(&mut sender, &status_event).await.is_err() {
        return;
    }

    if let Some(latest) = runtime.latest_score.read().await.clone() {
        let score_event = ServerEvent::Score(latest);
        if send_event(&mut sender, &score_event).await.is_err() {
            return;
        }
    }

    let runtime_for_receive = Arc::clone(&runtime);
    let receive_task = tokio::spawn(async move {
        while let Some(message) = receiver.next().await {
            match message {
                Ok(Message::Close(_)) => break,
                Ok(Message::Text(txt)) => {
                    if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&txt) {
                        tracing::info!("[WS] received client command: ({:?}, {:?})", cmd.x_pos, cmd.y_pos);

                        if let (Some(x), Some(y)) = (cmd.x_pos, cmd.y_pos) {
                            tracing::debug!("[WS] parsing coordinates: x={}, y={}", x, y);
                            let impact_point = Point::new(x, y);

                            match dart_game::calculate_score_for_impact_point(impact_point) {
                                Ok(payload) => {
                                    // Save latest score
                                    {
                                        let mut latest =
                                            runtime_for_receive.latest_score.write().await;
                                        *latest = Some(payload.clone());
                                    }

                                    // Check if a game is active
                                    let has_game =
                                        runtime_for_receive.active_game.read().await.is_some();
                                    tracing::debug!("[WS] active game session: {}", has_game);

                                    if has_game {
                                        // Derive ShotResult from the impact coordinates
                                        let shot_result = {
                                            let mut score = Score::new(Point::new(
                                                payload.impact_x,
                                                payload.impact_y,
                                            ));
                                            let sr = score.get_shortresult();
                                            tracing::debug!(
                                                "[WS] derived ShotResult: sector={}, multiplier={}",
                                                sr.sector,
                                                sr.multiplier,
                                            );
                                            sr
                                        };

                                        // Apply the shot to the active game session
                                        let mut game_state: Option<GameStatePayload> = None;
                                        let mut game_over: Option<(usize, String)> = None;

                                        {
                                            let mut game =
                                                runtime_for_receive.active_game.write().await;
                                            if let Some(ref mut session) = *game {
                                                let outcome = session.apply_shot(shot_result);
                                                let remaining = session.players[session.current_player].data;
                                                tracing::info!(
                                                    "[WS] shot applied — remaining={}, last_score={}, bust={}, turn_over={}, game_over={}",
                                                    remaining,
                                                    outcome.score,
                                                    outcome.is_bust,
                                                    outcome.turn_over,
                                                    outcome.game_over,
                                                );
                                                if let Some(msg) = &outcome.message {
                                                    tracing::info!("[WS] shot message: {}", msg);
                                                }

                                                game_state =
                                                    Some(game_state_from_session(session));

                                                if outcome.game_over {
                                                    let winner_idx = match session.phase {
                                                        dart_game::game::GamePhase::Finished(
                                                            i,
                                                        ) => i,
                                                        _ => session.current_player,
                                                    };
                                                    game_over = Some((
                                                        winner_idx,
                                                        session.players[winner_idx]
                                                            .name
                                                            .clone(),
                                                    ));
                                                }
                                            }
                                        }

                                        // Broadcast outside the write lock
                                        if let Some(state) = &game_state {
                                            tracing::info!(
                                                "[WS] broadcasting GameState — current_player={}, dart={}/3",
                                                state.current_player,
                                                state.current_dart + 1,
                                            );
                                            emit_event(
                                                &runtime_for_receive,
                                                ServerEvent::GameState(state.clone()),
                                            );
                                        }
                                        if let Some((idx, name)) = &game_over {
                                            tracing::info!(
                                                "[WS] 🏆 Game over! Winner: {} (player #{})",
                                                name,
                                                idx,
                                            );
                                            emit_event(
                                                &runtime_for_receive,
                                                ServerEvent::GameOver {
                                                    winner_index: *idx,
                                                    winner_name: name.clone(),
                                                },
                                            );
                                        }
                                    } else {
                                        tracing::debug!("[WS] no active game, broadcasting raw score");
                                        emit_event(
                                            &runtime_for_receive,
                                            ServerEvent::Score(payload),
                                        );
                                    }
                                }
                                Err(error) => {
                                    tracing::error!("[WS] score calculation failed: {}", error);
                                    emit_event(&runtime_for_receive, ServerEvent::Error(error));
                                }
                            }
                        } else {
                            tracing::warn!("[WS] received command with no coordinates: {:?}", cmd);
                        }
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });

    loop {
        match rx.recv().await {
            Ok(event) => {
                if send_event(&mut sender, &event).await.is_err() {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::error!("ws client lagged, skipped {} events", skipped);
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }

    receive_task.abort();
}

async fn send_event(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: &ServerEvent,
) -> Result<(), ()> {
    let text = serde_json::to_string(event).map_err(|_| ())?;
    sender.send(Message::Text(text.into())).await.map_err(|_| ())
}