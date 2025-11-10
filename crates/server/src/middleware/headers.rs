use axum::body::Body;
use axum::http::Request;
use axum::{http::HeaderValue, middleware::Next, response::Response};
use messaging_core::Config;
use std::sync::Arc;

/// Middleware: apply standard security headers if enabled in config.
/// Headers added:
/// - X-Frame-Options: DENY
/// - X-Content-Type-Options: nosniff
/// - Referrer-Policy: no-referrer
/// - Content-Security-Policy: default-src <cfg.csp_default_src>
pub async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let cfg = req.extensions().get::<Arc<Config>>().cloned();
    let mut resp = next.run(req).await;
    if let Some(cfg) = cfg {
        if cfg.security_headers_enabled {
            resp.headers_mut().insert(
                axum::http::header::HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );
            resp.headers_mut().insert(
                axum::http::header::HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
            resp.headers_mut().insert(
                axum::http::header::HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("no-referrer"),
            );
            let csp_value = format!("default-src {}", cfg.csp_default_src);
            if let Ok(hv) = HeaderValue::from_str(&csp_value) {
                resp.headers_mut().insert(
                    axum::http::header::HeaderName::from_static("content-security-policy"),
                    hv,
                );
            }
        }
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::{routing::get, Router};
    use messaging_core::Config;
    use std::sync::Arc;
    use tower::ServiceExt; // for oneshot

    #[tokio::test]
    async fn headers_added_when_enabled() {
        let mut cfg = Config::load().expect("cfg");
        cfg.security_headers_enabled = true;
        let shared = Arc::new(cfg);
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(security_headers))
            .layer(tower::util::MapRequestLayer::new(
                move |mut r: Request<_>| {
                    r.extensions_mut().insert(shared.clone());
                    r
                },
            ));
        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(resp.headers().get("x-frame-options").is_some());
        assert!(resp.headers().get("content-security-policy").is_some());
    }
}
