use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::executor::{AgentEvent, AgentRegistry};

use super::server::AppState;

#[derive(Serialize)]
struct MaestroEvent {
    event_type: String,
    scope: String,
    data: serde_json::Value,
}

impl MaestroEvent {
    fn from_agent_event(event: &AgentEvent) -> Self {
        match event {
            AgentEvent::Output(e) => MaestroEvent {
                event_type: "agent-output".to_string(),
                scope: e.workspace_id.clone(),
                data: serde_json::to_value(e).unwrap_or(serde_json::Value::Null),
            },
            AgentEvent::Exit(e) => MaestroEvent {
                event_type: "agent-exit".to_string(),
                scope: e.workspace_id.clone(),
                data: serde_json::to_value(e).unwrap_or(serde_json::Value::Null),
            },
        }
    }
}

pub async fn ws_events_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rx = state.event_bus.subscribe();
    ws.on_upgrade(move |socket| handle_ws_events(socket, rx))
}

async fn handle_ws_events(mut socket: WebSocket, mut rx: broadcast::Receiver<AgentEvent>) {
    loop {
        tokio::select! {
            event_result = rx.recv() => {
                match event_result {
                    Ok(event) => {
                        let maestro_event = MaestroEvent::from_agent_event(&event);
                        let json = match serde_json::to_string(&maestro_event) {
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
    let rx = state.event_bus.subscribe();
    let registry = state.registry.clone();
    ws.on_upgrade(move |socket| handle_ws_agent(socket, workspace_id, rx, registry))
}

async fn handle_ws_agent(
    mut socket: WebSocket,
    workspace_id: String,
    mut rx: broadcast::Receiver<AgentEvent>,
    registry: Arc<AgentRegistry>,
) {
    loop {
        tokio::select! {
            event_result = rx.recv() => {
                match event_result {
                    Ok(event) => {
                        let maestro_event = MaestroEvent::from_agent_event(&event);
                        if maestro_event.scope != workspace_id
                            || !maestro_event.event_type.starts_with("agent-")
                        {
                            continue;
                        }
                        let json = match serde_json::to_string(&maestro_event) {
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
