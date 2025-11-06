use axum::{body::Body, http::Request, middleware::Next, response::Response};
use std::time::Instant;

fn client_ip(req: &Request<Body>) -> String {
    const XFF: &str = "x-forwarded-for";
    const X_REAL_IP: &str = "x-real-ip";
    if let Some(v) = req.headers().get(XFF) {
        if let Ok(s) = v.to_str() {
            if let Some(first) = s.split(',').next() {
                let ip = first.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }
    if let Some(v) = req.headers().get(X_REAL_IP) {
        if let Ok(s) = v.to_str() {
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Log each request with method, path, status, duration, and client IP.
pub async fn log_requests(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let ip = client_ip(&req);
    let start = Instant::now();

    let resp = next.run(req).await;

    let status = resp.status().as_u16();
    let took_ms = start.elapsed().as_millis();

    tracing::info!(
        target = "server",
        event = "http_request",
        %method,
        path = %path,
        status = status,
        duration_ms = took_ms,
        client_ip = %ip,
        "handled request"
    );

    resp
}
