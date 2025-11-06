use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware as axmw;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};
use messaging_core::Config;
use serde::Serialize;
use std::future::Future;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

// Expose internal modules for middleware and types so they can be wired in later phases
pub mod config;
pub mod errors;
pub mod types;
pub mod middleware {
    pub mod accept;
    pub mod circuit_breaker;
    pub mod content_type;
    pub mod idempotency;
    pub mod limits;
    pub mod logging;
    pub mod rate_limit;
}
pub mod queue {
    pub mod inbound_events;
}
pub mod state {
    pub mod idempotency;
}

use crate::config::ApiConfig;
use crate::middleware::circuit_breaker::{BreakerState, CircuitBreaker};
use crate::middleware::rate_limit::RateLimiter;
use crate::queue::inbound_events::InboundQueue;
use crate::state::idempotency::IdempotencyStore;

pub mod api {
    pub mod conversations;
    pub mod messages;
    pub mod webhooks;
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Clone)]
pub(crate) struct AppState {
    api: ApiConfig,
    rate: RateLimiter,
    breaker: CircuitBreaker,
    queue: InboundQueue,
    idempotency: IdempotencyStore,
}

fn build_router(health_path: &str, state: AppState) -> Router {
    let path = health_path.to_string();
    Router::new()
        .route(&path, get(health_handler))
        // API routes expected by bin/test.sh (no version prefix)
        .route(
            "/api/messages/sms",
            axum::routing::post(api::messages::post_sms),
        )
        .route(
            "/api/messages/email",
            axum::routing::post(api::messages::post_email),
        )
        .route(
            "/api/webhooks/sms",
            axum::routing::post(api::webhooks::post_sms),
        )
        .route(
            "/api/webhooks/email",
            axum::routing::post(api::webhooks::post_email),
        )
        .route(
            "/api/conversations",
            get(api::conversations::list_conversations),
        )
        .route(
            "/api/conversations/{id}/messages",
            get(api::conversations::list_messages),
        )
        // Global middleware for this phase; specific routes will be added in later phases
        .layer(axmw::from_fn(
            crate::middleware::accept::enforce_json_accept,
        ))
        .layer(axmw::from_fn(
            crate::middleware::content_type::enforce_json_content_type,
        ))
        .layer(axmw::from_fn(extract_idempotency_layer))
        .layer(axmw::from_fn_with_state(
            state.clone(),
            circuit_breaker_layer,
        ))
        .layer(axmw::from_fn_with_state(state.clone(), rate_limit_ip_layer))
        .layer(crate::middleware::limits::body_limit(
            state.api.max_body_bytes,
        ))
        // Outermost: request logging
        .layer(axmw::from_fn(crate::middleware::logging::log_requests))
        .with_state(state)
}

async fn health_handler() -> Json<Health> {
    tracing::info!(target: "server", event = "health", "health check");
    Json(Health { status: "ok" })
}

pub async fn run_server(
    config: Arc<Config>,
) -> Result<(tokio::task::JoinHandle<()>, SocketAddr), String> {
    // Build shared state
    let (queue, mut rx) = InboundQueue::new(1024);
    // Drain queue in background (stub)
    let _drain = tokio::spawn(async move {
        while let Some(_evt) = rx.recv().await {
            // TODO: persist/dispatch
        }
    });
    let api_cfg = ApiConfig::default();
    let state = AppState {
        rate: RateLimiter::new(
            api_cfg.rate_limit_per_ip_per_min,
            api_cfg.rate_limit_per_sender_per_min,
        ),
        breaker: CircuitBreaker::new(api_cfg.breaker_error_threshold, api_cfg.breaker_open_secs),
        queue,
        idempotency: IdempotencyStore::new(2 * 60 * 60), // 2 hours
        api: api_cfg,
    };
    let router = build_router(&config.health_path, state);

    // Bind to 127.0.0.1 for tests; for production US1 scope, bind on 0.0.0.0
    let bind_addr: SocketAddr = ([0, 0, 0, 0], config.port).into();
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|e| format!("failed to bind: {e}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("failed to read local addr: {e}"))?;

    tracing::info!(target: "server", event = "startup", %local_addr, health_path = %config.health_path, "listening");

    let server = axum::serve(listener, router.into_make_service());
    let handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("server error: {e}");
        }
    });
    Ok((handle, local_addr))
}

pub async fn run_server_with_shutdown<F>(
    config: Arc<Config>,
    shutdown: F,
) -> Result<(tokio::task::JoinHandle<()>, SocketAddr), String>
where
    F: Future<Output = ()> + Send + 'static,
{
    // Build shared state
    let (queue, mut rx) = InboundQueue::new(1024);
    // Drain queue in background (stub)
    let _drain = tokio::spawn(async move {
        while let Some(_evt) = rx.recv().await {
            // TODO: persist/dispatch
        }
    });
    let api_cfg = ApiConfig::default();
    let state = AppState {
        rate: RateLimiter::new(
            api_cfg.rate_limit_per_ip_per_min,
            api_cfg.rate_limit_per_sender_per_min,
        ),
        breaker: CircuitBreaker::new(api_cfg.breaker_error_threshold, api_cfg.breaker_open_secs),
        queue,
        idempotency: IdempotencyStore::new(2 * 60 * 60),
        api: api_cfg,
    };
    let router = build_router(&config.health_path, state);

    let bind_addr: SocketAddr = ([0, 0, 0, 0], config.port).into();
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|e| format!("failed to bind: {e}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("failed to read local addr: {e}"))?;

    tracing::info!(target: "server", event = "startup", %local_addr, health_path = %config.health_path, "listening");

    let server = axum::serve(listener, router.into_make_service()).with_graceful_shutdown(shutdown);

    let handle = tokio::spawn(async move {
        match server.await {
            Ok(()) => {
                tracing::info!(target: "server", event = "shutdown", "server shutdown complete");
            }
            Err(e) => {
                eprintln!("server error: {e}");
            }
        }
    });
    Ok((handle, local_addr))
}

// ---------- Middleware glue (Phase 2) ----------

/// Cheap helper: pull client IP from common proxy headers; fall back to unknown
fn client_ip_from_headers(req: &Request<Body>) -> String {
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

async fn rate_limit_ip_layer(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if matches!(req.method().as_str(), "POST" | "PUT" | "PATCH" | "GET") {
        let ip = client_ip_from_headers(&req);
        if !state.rate.allow_ip(&ip) {
            let (status, body) = crate::errors::too_many_requests("Too many requests from IP");
            let mut resp = (status, body).into_response();
            resp.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                axum::http::HeaderValue::from_static("60"),
            );
            return resp;
        }
    }
    next.run(req).await
}

async fn circuit_breaker_layer(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if state.breaker.before_request() == BreakerState::Open {
        let (status, body) = crate::errors::service_unavailable("Temporarily unavailable");
        let mut resp = (status, body).into_response();
        let secs = state.breaker.recovery_timeout.as_secs().to_string();
        if let Ok(v) = axum::http::HeaderValue::from_str(&secs) {
            resp.headers_mut()
                .insert(axum::http::header::RETRY_AFTER, v);
        }
        return resp;
    }
    let response = next.run(req).await;
    // Update breaker based on response status
    let status = response.status();
    if status.is_server_error() {
        state.breaker.record_failure();
    } else if status.is_success() || status == StatusCode::ACCEPTED || status == StatusCode::CREATED
    {
        state.breaker.record_success();
    }
    response
}

// Adapter to reuse existing idempotency extractor that doesn't take state
async fn extract_idempotency_layer(req: Request<Body>, next: Next) -> Response {
    crate::middleware::idempotency::extract_idempotency_key(req, next).await
}
