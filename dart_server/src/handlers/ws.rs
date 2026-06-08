// ── External crates ──
use axum::{
    extract::{State, ws::WebSocketUpgrade},
    response::IntoResponse,
};

// ── Internal modules ──
use crate::connection::ws_connection;
use crate::runtime::AppState;

/// Upgrades the HTTP connection to WebSocket and hands off to the connection handler.
pub(crate) async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connection(socket, state))
}
