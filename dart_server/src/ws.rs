use crate::runtime::{AppState, ServerEvent};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::Arc;

use crate::handlers::{emit_event, status_from_runtime};

use dartdetect::{
    dart_core::Point,
    dart_scoring::processor,
};

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
                        tracing::info!(
                            "[WS] received client command: ({:?}, {:?})",
                            cmd.x_pos,
                            cmd.y_pos
                        );

                        if let (Some(x), Some(y)) = (cmd.x_pos, cmd.y_pos) {
                            tracing::debug!("[WS] parsing coordinates: x={}, y={}", x, y);
                            let impact_point = Point::new(x, y);

                            match processor::calculate_score_for_impact_point(impact_point) {
                                Ok(score) => {
                                    crate::game_manager::process_impact(
                                        &runtime_for_receive,
                                        x,
                                        y,
                                        score,
                                    )
                                    .await;
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

                            match processor::calculate_score_for_sensors_timings(t1, t2, t3, t4) {
                                Ok((impact_point, score)) => {
                                    crate::game_manager::process_impact(
                                        &runtime_for_receive,
                                        impact_point.x,
                                        impact_point.y,
                                        score,
                                    )
                                    .await;
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
