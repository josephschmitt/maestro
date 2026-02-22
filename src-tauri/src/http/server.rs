use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    middleware::{self, Next},
    Router,
};
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

use crate::commands::config::ConfigState;
use crate::events::EventBus;
use crate::executor::AgentRegistry;
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

pub async fn start_http_server(app_state: AppState, bind_addr: String) -> Result<(), String> {
    let server_url = app_state.server_url.clone();
    let frontend_dir = resolve_frontend_dir();

    let router = build_router(app_state, &frontend_dir);

    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("Failed to bind HTTP server to {bind_addr}: {e}"))?;

    eprintln!("[http] Server listening on http://{server_url}");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("[http] Server error: {e}");
        }
    });

    Ok(())
}

fn build_router(app_state: AppState, frontend_dir: &str) -> Router {
    let server_url = app_state.server_url.clone();

    let index_path = format!("{frontend_dir}/index.html");

    let serve_dir = ServeDir::new(frontend_dir)
        .not_found_service(ServeFile::new(&index_path));

    Router::new()
        .merge(api_routes())
        .fallback_service(serve_dir)
        .layer(middleware::from_fn(move |req, next| {
            let url = server_url.clone();
            inject_html_middleware(req, next, url)
        }))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

async fn inject_html_middleware(
    req: Request<Body>,
    next: Next,
    server_url: String,
) -> Response<Body> {
    let response = next.run(req).await;

    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/html"))
        .unwrap_or(false);

    if !is_html {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap();
        }
    };

    let html = String::from_utf8_lossy(&bytes);
    let script_tag = format!(
        r#"<script>window.__MAESTRO_HTTP_URL__ = "{}";</script>"#,
        server_url
    );

    let injected = if html.contains("</head>") {
        html.replace("</head>", &format!("{script_tag}</head>"))
    } else {
        format!("{script_tag}{html}")
    };

    let mut new_parts = parts;
    let body_bytes = injected.into_bytes();
    new_parts.headers.insert(
        header::CONTENT_LENGTH,
        header::HeaderValue::from(body_bytes.len()),
    );

    Response::from_parts(new_parts, Body::from(body_bytes))
}

fn resolve_frontend_dir() -> String {
    // In dev mode, use the SvelteKit build output directory
    let dev_path = std::env::current_dir()
        .ok()
        .map(|p| p.join("../build"))
        .filter(|p| p.exists());

    if let Some(path) = dev_path {
        return path.to_string_lossy().to_string();
    }

    // In production, use the Tauri resource directory
    // Tauri bundles frontendDist into the resource dir
    #[cfg(not(debug_assertions))]
    {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                // macOS: .app/Contents/Resources
                let resource_dir = parent.join("../Resources");
                if resource_dir.exists() {
                    return resource_dir.to_string_lossy().to_string();
                }
            }
        }
    }

    // Fallback to ../build relative to working dir
    "../build".to_string()
}
