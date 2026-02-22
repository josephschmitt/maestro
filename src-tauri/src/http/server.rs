use std::net::SocketAddr;
use std::sync::Arc;

use axum::middleware;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::commands::config::ConfigState;
use crate::config::global::HttpServerConfig;
use crate::executor::{AgentRegistry, EventBus};
use crate::ipc::server::IpcServer;

use super::auth::auth_middleware;
use super::routes::api_routes;
use super::websocket::{ws_agent_handler, ws_events_handler};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ConfigState>,
    pub registry: Arc<AgentRegistry>,
    pub ipc: Arc<IpcServer>,
    pub event_bus: Arc<EventBus>,
    pub server_url: String,
    pub http_config: Arc<HttpServerConfig>,
}

impl AppState {
    pub fn new(
        config: ConfigState,
        registry: Arc<AgentRegistry>,
        ipc: Arc<IpcServer>,
        event_bus: EventBus,
        http_config: HttpServerConfig,
    ) -> Self {
        let server_url = format!("http://{}:{}", http_config.bind_address, http_config.port);
        Self {
            config: Arc::new(config),
            registry,
            ipc,
            event_bus: Arc::new(event_bus),
            server_url,
            http_config: Arc::new(http_config),
        }
    }
}

pub async fn start_http_server(state: AppState) -> Result<(), String> {
    let http_config = state.http_config.clone();
    let bind_address = http_config.bind_address.clone();
    let port = http_config.port;

    let cors = if http_config.requires_auth() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::permissive()
    };

    let app = Router::new()
        .nest("/api", api_routes())
        .route("/ws/events", get(ws_events_handler))
        .route("/ws/agent/:workspace_id", get(ws_agent_handler))
        .layer(middleware::from_fn(move |req, next| {
            let config = http_config.clone();
            auth_middleware(config, req, next)
        }))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", bind_address, port)
        .parse()
        .map_err(|e| format!("Invalid bind address: {e}"))?;
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind HTTP server: {e}"))?;

    eprintln!("[http] Server listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("HTTP server error: {e}"))
}
