use axum::http::header::{HeaderValue, ACCEPT};
use axum::{
    body::Body, http::Request, middleware::Next, response::IntoResponse, response::Response,
};

use crate::errors;

fn accepts_json(value: &HeaderValue) -> bool {
    // Accept: application/json, */*;q=0.8, etc.
    match value.to_str() {
        Ok(s) => {
            let s = s.to_ascii_lowercase();
            s.split(',')
                .map(|part| part.trim())
                .any(|part| part.starts_with("application/json") || part == "*/*")
        }
        Err(_) => false,
    }
}

/// Enforce JSON responses for GET-like endpoints when Accept is specified.
pub async fn enforce_json_accept(req: Request<Body>, next: Next) -> Response {
    if matches!(req.method().as_str(), "GET" | "HEAD") {
        if let Some(v) = req.headers().get(ACCEPT) {
            if !accepts_json(v) {
                let (status, body) = errors::not_acceptable();
                return (status, body).into_response();
            }
        }
    }
    next.run(req).await
}
