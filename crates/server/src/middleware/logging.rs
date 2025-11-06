use axum::{body::Body, http::Request, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

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
pub async fn log_requests(mut req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let ip = client_ip(&req);
    let start = Instant::now();

    // Correlation / request ID: honor inbound header or generate new
    const HDR_REQUEST_ID: &str = "x-request-id";
    let correlation_id = req
        .headers()
        .get(HDR_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Basic redaction sample: capture header names but not values for sensitive keys
    const SENSITIVE: [&str; 3] = ["authorization", "x-api-key", "cookie"];
    let mut header_count = 0usize;
    let mut sensitive_present = Vec::new();
    for (name, _val) in req.headers().iter() {
        header_count += 1;
        let n = name.as_str().to_ascii_lowercase();
        if SENSITIVE.contains(&n.as_str()) {
            sensitive_present.push(n);
        }
    }

    // Attach correlation id to request extensions for downstream use
    req.extensions_mut().insert(correlation_id.clone());

    let mut resp = next.run(req).await;

    // Propagate correlation id back to caller
    if let Ok(hv) = axum::http::HeaderValue::from_str(&correlation_id) {
        resp.headers_mut().insert(
            axum::http::header::HeaderName::from_static(HDR_REQUEST_ID),
            hv,
        );
    }

    let status = resp.status().as_u16();
    let took_us = start.elapsed().as_micros();

    tracing::info!(
        target = "server",
        event = "http_request",
        %method,
        path = %path,
        status = status,
        duration_us = took_us,
        client_ip = %ip,
        correlation_id = %correlation_id,
        header_count,
        sensitive_headers = %format!("[{}]", sensitive_present.join(",")),
        "handled request"
    );

    resp
}
