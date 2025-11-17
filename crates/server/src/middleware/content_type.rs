use axum::http::header::{HeaderValue, CONTENT_TYPE};
use axum::{
    body::Body, http::Request, middleware::Next, response::IntoResponse, response::Response,
};

use crate::errors;

/// Enforce `Content-Type: application/json` for POST-like requests.
pub async fn enforce_json_content_type(req: Request<Body>, next: Next) -> Response {
    // Only enforce for methods that carry bodies typically
    let method = req.method().clone();
    if matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
        let is_json = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v: &HeaderValue| v.to_str().ok())
            .map(|s| s.to_ascii_lowercase().starts_with("application/json"))
            .unwrap_or(false);
        if !is_json {
            let (status, body) = errors::unsupported_media_type();
            return (status, body).into_response();
        }
    }
    next.run(req).await
}
