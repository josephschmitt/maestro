use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::executor::{AgentRegistry, MaestroEvent};

use super::server::AppState;

#[derive(Serialize)]
struct WebSocketEvent {
    event_type: String,
    scope: Option<String>,
    data: serde_json::Value,
}

impl WebSocketEvent {
    fn from_maestro_event(event: &MaestroEvent) -> Self {
        WebSocketEvent {
            event_type: event.event_type().to_string(),
            scope: event.scope().map(|s| s.to_string()),
            data: serde_json::to_value(event).unwrap_or(serde_json::Value::Null),
        }
    }
}

pub async fn ws_events_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rx = state.event_bus.subscribe_maestro();
    ws.on_upgrade(move |socket| handle_ws_events(socket, rx))
}

async fn handle_ws_events(mut socket: WebSocket, mut rx: broadcast::Receiver<MaestroEvent>) {
    loop {
        tokio::select! {
            event_result = rx.recv() => {
                match event_result {
                    Ok(event) => {
                        let ws_event = WebSocketEvent::from_maestro_event(&event);
                        let json = match serde_json::to_string(&ws_event) {
                            Ok(j) => j,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

pub async fn ws_agent_handler(
    ws: WebSocketUpgrade,
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rx = state.event_bus.subscribe_maestro();
    let registry = state.registry.clone();
    ws.on_upgrade(move |socket| handle_ws_agent(socket, workspace_id, rx, registry))
}

async fn handle_ws_agent(
    mut socket: WebSocket,
    workspace_id: String,
    mut rx: broadcast::Receiver<MaestroEvent>,
    registry: Arc<AgentRegistry>,
) {
    loop {
        tokio::select! {
            event_result = rx.recv() => {
                match event_result {
                    Ok(event) => {
                        let event_type = event.event_type();
                        let scope = event.scope();
                        if scope != Some(workspace_id.as_str())
                            || !event_type.starts_with("agent-")
                        {
                            continue;
                        }
                        let ws_event = WebSocketEvent::from_maestro_event(&event);
                        let json = match serde_json::to_string(&ws_event) {
                            Ok(j) => j,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Some(tx) = registry.get_stdin_tx(&workspace_id) {
                            let _ = tx.send(text).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
