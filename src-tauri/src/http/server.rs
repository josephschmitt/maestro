use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use crate::commands::config::ConfigState;
use crate::executor::{AgentRegistry, EventBus};
use crate::ipc::server::IpcServer;

use super::routes::api_routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ConfigState>,
    pub registry: Arc<AgentRegistry>,
    pub ipc: Arc<IpcServer>,
    pub event_bus: Arc<EventBus>,
    pub server_url: String,
}

impl AppState {
    pub fn new(
        config: ConfigState,
        registry: Arc<AgentRegistry>,
        ipc: Arc<IpcServer>,
        event_bus: EventBus,
        port: u16,
    ) -> Self {
        Self {
            config: Arc::new(config),
            registry,
            ipc,
            event_bus: Arc::new(event_bus),
            server_url: format!("http://127.0.0.1:{port}"),
        }
    }
}

pub async fn start_http_server(state: AppState, port: u16) -> Result<(), String> {
    let app = Router::new()
        .nest("/api", api_routes())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind HTTP server: {e}"))?;

    eprintln!("[http] Server listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("HTTP server error: {e}"))
}
