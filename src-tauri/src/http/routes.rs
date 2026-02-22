use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use super::server::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new().route("/api/health", get(health))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
