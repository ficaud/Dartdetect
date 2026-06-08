use crate::runtime::{AppState, ServerEvent, SimulationRuntime};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::Arc;

// Handlers import
use crate::handlers::{emit_event, status_from_runtime};

// Dart detect import
use dartdetect::{dart_core::Point, dart_scoring::processor};

/// Incoming command from a WS client — JSON `{ x_pos, y_pos }` or `{ t1, t2, t3, t4 }`.
#[derive(serde::Deserialize, Debug)]
struct ClientCommand {
    x_pos: Option<f64>,
    y_pos: Option<f64>,
    t1: Option<f64>,
    t2: Option<f64>,
    t3: Option<f64>,
    t4: Option<f64>,
}

/// WebSocket connection handler:
///
/// listens for incoming client commands, processes them to calculate scores and update game state,
/// and broadcasts events back to the client.
///
/// # Arguments
/// - `socket`: the WebSocket connection to the client, used to send and receive messages
/// - `state`: the shared application state containing the simulation runtime, used to access and update the game state and broadcast events
///
/// This function performs the following steps:
/// 1. Subscribes to the runtime's event broadcast channel to receive updates about scores, game state, and simulation status.
///
/// 2. Sends the initial simulation status and latest score to the client upon connection, so they have the current state of the game immediately.
///
/// 3. Spawns a task to listen for incoming messages from the client, which are expected to be JSON commands containing either impact coordinates or raw sensor timings.
///    It processes these commands to calculate scores and update the game state accordingly.
///
/// 4. Enters a loop to listen for events from the runtime and sends them to the client as they occur,
///    allowing the client to stay updated with the latest game state and scores.
///
/// 5. Handles client disconnection and errors gracefully by breaking the loop and aborting the receive task when necessary.
pub(crate) async fn ws_connection(socket: WebSocket, state: AppState) {
    let runtime = Arc::clone(&state.runtime);
    let mut rx = runtime.events_tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    // Send initial status and latest score to the client upon connection.
    let status_event = ServerEvent::Status(status_from_runtime(&runtime));
    if send_event(&mut sender, &status_event).await.is_err() {
        return;
    }

    // Send latest score if available (e.g. if client connects in the middle of a game)
    if let Some(latest) = runtime.latest_score.read().await.clone() {
        let score_event = ServerEvent::Score(latest);
        if send_event(&mut sender, &score_event).await.is_err() {
            return;
        }
    }

    // Spawn a task to listen for incoming messages from the client, parse them as commands, calculate scores, and update the game state.
    let runtime_for_receive = Arc::clone(&runtime);
    let receive_task = tokio::spawn(async move {
        /// Common helper: apply impact via game manager and broadcast resulting events.
        async fn handle_impact(runtime: &SimulationRuntime, x: f64, y: f64, score: u32) {
            let events = crate::game_manager::process_impact(runtime, x, y, score).await;
            for event in events {
                emit_event(runtime, event);
            }
        }

        // Listen for incoming messages from the client, parse them as commands, calculate scores, and update the game state.
        while let Some(message) = receiver.next().await {
            match message {
                Ok(Message::Close(_)) => break,
                Ok(Message::Text(txt)) => {
                    if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&txt) {
                        tracing::info!(
                            "[WS] received client command: ({:?}, {:?})",
                            cmd.x_pos,
                            cmd.y_pos
                        );

                        // If the commande contains coordinates, calculate score from them.
                        //
                        // Otherwise, if it contains raw timings, calculate impact point and score from them.
                        //
                        // If neither, ignore the command
                        if let (Some(x), Some(y)) = (cmd.x_pos, cmd.y_pos) {
                            tracing::debug!("[WS] parsing coordinates: x={}, y={}", x, y);

                            // Calculate score from impact point, then handle impact.
                            match processor::calculate_score_for_impact_point(Point::new(x, y)) {
                                Ok(score) => {
                                    // Call the halper that handle the common flow after an impact:
                                    // save score, apply shot to game if active, emit events.
                                    handle_impact(&runtime_for_receive, x, y, score).await
                                }
                                Err(error) => {
                                    tracing::error!("[WS] score calculation failed: {}", error);
                                    emit_event(&runtime_for_receive, ServerEvent::Error(error));
                                }
                            }
                        } else if let (Some(t1), Some(t2), Some(t3), Some(t4)) =
                            (cmd.t1, cmd.t2, cmd.t3, cmd.t4)
                        {
                            tracing::info!(
                                "[WS] received raw timings: t1={}, t2={}, t3={}, t4={}",
                                t1,
                                t2,
                                t3,
                                t4
                            );

                            // Calculate impact point and score from raw timings, then handle impact.
                            match processor::calculate_score_for_sensors_timings(t1, t2, t3, t4) {
                                Ok((impact_point, score)) => {
                                    // Call the halper that handle the common flow after an impact:
                                    // save score, apply shot to game if active, emit events.
                                    handle_impact(
                                        &runtime_for_receive,
                                        impact_point.x,
                                        impact_point.y,
                                        score,
                                    )
                                    .await
                                }
                                Err(error) => {
                                    tracing::error!("[WS] timing calculation failed: {}", error);
                                }
                            }
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

/// Helper function to send a ServerEvent to the client via the WebSocket connection.
///
/// # Arguments
/// - `sender`: the split sink of the WebSocket connection used to send messages to the client
/// - `event`: the ServerEvent to be sent to the client, which will be serialized to JSON before sending
///
/// # Returns
/// - `Result<(), ()>`: Ok if the event was sent successfully, EErr otherwise
async fn send_event(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: &ServerEvent,
) -> Result<(), ()> {
    let text = serde_json::to_string(event).map_err(|_| ())?;
    sender
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}
