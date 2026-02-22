use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::config::global::HttpServerConfig;

pub async fn auth_middleware(
    config: Arc<HttpServerConfig>,
    request: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    if !config.requires_auth() {
        return Ok(next.run(request).await);
    }

    let expected_token = &config.auth_token;
    if expected_token.is_empty() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let token = extract_token(&request);

    match token {
        Some(t) if t == expected_token => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

fn extract_token(request: &Request) -> Option<&str> {
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token);
            }
        }
    }

    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some(token) = pair.strip_prefix("token=") {
                return Some(token);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request_with_header(token: &str) -> Request {
        Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap()
    }

    fn make_request_with_query(token: &str) -> Request {
        Request::builder()
            .uri(format!("/test?token={}", token))
            .body(Body::empty())
            .unwrap()
    }

    fn make_request_without_auth() -> Request {
        Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn test_extract_token_from_header() {
        let request = make_request_with_header("my-secret-token");
        assert_eq!(extract_token(&request), Some("my-secret-token"));
    }

    #[test]
    fn test_extract_token_from_query() {
        let request = make_request_with_query("my-secret-token");
        assert_eq!(extract_token(&request), Some("my-secret-token"));
    }

    #[test]
    fn test_extract_token_none() {
        let request = make_request_without_auth();
        assert_eq!(extract_token(&request), None);
    }

    #[test]
    fn test_extract_token_query_with_other_params() {
        let request = Request::builder()
            .uri("/test?foo=bar&token=abc123&baz=qux")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_token(&request), Some("abc123"));
    }
}
