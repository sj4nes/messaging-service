use axum::http::header::HeaderName;
use axum::{body::Body, http::Request, middleware::Next, response::Response};

/// Header name used to carry idempotency key for POST endpoints
const IDEMPOTENCY_HEADER_STR: &str = "idempotency-key";

#[derive(Debug, Clone)]
pub struct IdempotencyKey(pub String);

/// Middleware that extracts an Idempotency-Key header and stores it in request extensions.
pub async fn extract_idempotency_key(mut req: Request<Body>, next: Next) -> Response {
    let header = HeaderName::from_static(IDEMPOTENCY_HEADER_STR);
    let key: Option<String> = req
        .headers()
        .get(&header)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    if let Some(k) = key {
        req.extensions_mut().insert(IdempotencyKey(k));
    }
    next.run(req).await
}
