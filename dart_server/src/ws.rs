use crate::runtime::{AppState, ServerEvent};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::{sync::Arc};
use axum::extract::ws::{Message, WebSocket};

use crate::handlers::status_from_runtime;
use crate::handlers::emit_event;
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
                        tracing::info!("received client command: {:?}", cmd);
                        // handle positional command or others here
                        if let (Some(x), Some(y)) = (cmd.x_pos, cmd.y_pos) {
                            let impact_point = dartdectec::dart_core::Point::new(x, y);
                            match dartdectec::dart_game::calculate_score_for_impact_point(impact_point) {
                                Ok(payload) => {
                                    {
                                        let mut latest = runtime_for_receive.latest_score.write().await;
                                        *latest = Some(payload.clone());
                                    }
                                    emit_event(&runtime_for_receive, ServerEvent::Score(payload));
                                }
                                Err(error) => {
                                    emit_event(&runtime_for_receive, ServerEvent::Error(error));
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

async fn send_event(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: &ServerEvent,
) -> Result<(), ()> {
    let text = serde_json::to_string(event).map_err(|_| ())?;
    sender.send(Message::Text(text.into())).await.map_err(|_| ())
}