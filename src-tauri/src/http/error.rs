use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Internal(err)
    }
}

pub fn extract_arg<T: serde::de::DeserializeOwned>(
    args: &serde_json::Value,
    key: &str,
) -> Result<T, AppError> {
    if args.get(key).is_none() || args[key].is_null() {
        return Err(AppError::BadRequest(format!("missing required arg: {key}")));
    }
    serde_json::from_value(args[key].clone())
        .map_err(|_| AppError::BadRequest(format!("invalid arg: {key}")))
}

pub fn extract_optional_arg<T: serde::de::DeserializeOwned>(
    args: &serde_json::Value,
    key: &str,
) -> Result<Option<T>, AppError> {
    if args.get(key).is_none() || args[key].is_null() {
        return Ok(None);
    }
    Ok(Some(
        serde_json::from_value(args[key].clone())
            .map_err(|_| AppError::BadRequest(format!("invalid arg: {key}")))?,
    ))
}
