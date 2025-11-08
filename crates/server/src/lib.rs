use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware as axmw;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};
use messaging_core::Config;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
// (deduped) PgPoolOptions imported above
use std::future::Future;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

// Expose internal modules for middleware and types so they can be wired in later phases
pub mod config;
pub mod errors;
pub mod logging;
pub mod metrics;
pub mod snippet;
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
    pub mod outbound;
}
pub mod state {
    pub mod breakers;
    pub mod idempotency;
}

// New modules for provider mocks and stores (Feature 006)
pub mod providers {
    pub mod mock;
    // Feature 008 provider modules scaffolds
    pub mod common;
    pub mod email;
    pub mod registry;
    pub mod sms_mms; // shared helpers (Feature 008)
}
pub mod store {
    pub mod conversations;
    pub mod messages;
}
// DB-backed stores (Feature 007 scaffolding)
pub mod store_db {
    pub mod conversations;
    pub mod inbound_events;
    pub mod messages;
    pub mod normalize;
    pub mod seed;
}
pub mod worker {
    pub mod inbound;
}

use crate::config::ApiConfig;
use crate::middleware::circuit_breaker::{BreakerState, CircuitBreaker};
use crate::middleware::rate_limit::RateLimiter;
use crate::queue::inbound_events::InboundQueue;
use crate::state::idempotency::IdempotencyStore;

pub mod api {
    pub mod conversations;
    pub mod messages;
    pub mod provider_mock;
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
    db: Option<sqlx::PgPool>,
    // Feature 008: provider registry (per-channel routing)
    provider_registry: crate::providers::registry::ProviderRegistry,
    // Feature 008: per-provider circuit breakers
    provider_breakers: crate::state::breakers::ProviderBreakers,
    snippet_length: usize,
}

impl AppState {
    pub(crate) fn db(&self) -> Option<sqlx::PgPool> {
        self.db.clone()
    }

    pub(crate) fn snippet_len(&self) -> usize {
        self.snippet_length
    }

    pub(crate) fn inmemory_fallback_enabled(&self) -> bool {
        self.api.enable_inmemory_fallback
    }
}

fn build_router(health_path: &str, state: AppState) -> Router {
    let path = health_path.to_string();
    Router::new()
        .route(&path, get(health_handler))
        .route("/metrics", get(metrics_handler))
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
        .route(
            "/api/provider/mock/inbound",
            axum::routing::post(api::provider_mock::post_inbound),
        )
        .route(
            "/api/provider/mock/config",
            get(api::provider_mock::get_config).put(api::provider_mock::put_config),
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

async fn metrics_handler() -> Json<crate::metrics::MetricsSnapshot> {
    Json(crate::metrics::snapshot())
}

pub async fn run_server(
    config: Arc<Config>,
) -> Result<(tokio::task::JoinHandle<()>, SocketAddr), String> {
    // Build shared state
    let (queue, rx) = InboundQueue::new(1024);
    let api_cfg = ApiConfig::load();
    // Initialize deterministic provider RNG seeds (US3 T030/T033)
    crate::providers::common::init_rng_seeds(&api_cfg);
    let api_cfg_for_worker = api_cfg.clone();
    let brk_thresh = api_cfg.breaker_error_threshold;
    let brk_open_secs = api_cfg.breaker_open_secs;
    // Optionally create DB pool if DATABASE_URL is set
    let db_pool: Option<sqlx::PgPool> = match std::env::var("DATABASE_URL") {
        Ok(url) => match PgPoolOptions::new().max_connections(5).connect(&url).await {
            Ok(pool) => Some(pool),
            Err(e) => {
                tracing::warn!(target="server", event="db_pool_error", error=%e, "failed to create DB pool; continuing without DB");
                None
            }
        },
        Err(_) => None,
    };
    // Build provider registry and implementations (US1 wiring)
    let provider_registry = {
        use crate::providers::registry::{ChannelKind, Provider, ProviderRegistry};
        use std::sync::Arc;
        let mut reg = ProviderRegistry::new();
        let sms =
            Arc::new(crate::providers::sms_mms::SmsMmsMockProvider::new()) as Arc<dyn Provider>;
        let email =
            Arc::new(crate::providers::email::EmailMockProvider::new()) as Arc<dyn Provider>;
        reg.insert(ChannelKind::Sms, sms.clone());
        reg.insert(ChannelKind::Mms, sms);
        reg.insert(ChannelKind::Email, email);
        reg
    };

    let state = AppState {
        rate: RateLimiter::new(
            api_cfg.rate_limit_per_ip_per_min,
            api_cfg.rate_limit_per_sender_per_min,
        ),
        breaker: CircuitBreaker::new(api_cfg.breaker_error_threshold, api_cfg.breaker_open_secs),
        queue,
        idempotency: IdempotencyStore::new(2 * 60 * 60), // 2 hours
        api: api_cfg.clone(),
        db: db_pool.clone(),
        provider_registry,
        provider_breakers: {
            use crate::middleware::circuit_breaker::CircuitBreaker;
            let mut map = std::collections::HashMap::new();
            // Pre-create breakers for known providers; names align with metrics labels
            map.insert(
                crate::metrics::PROVIDER_LABEL_SMS_MMS.to_string(),
                CircuitBreaker::new(brk_thresh, brk_open_secs),
            );
            map.insert(
                crate::metrics::PROVIDER_LABEL_EMAIL.to_string(),
                CircuitBreaker::new(brk_thresh, brk_open_secs),
            );
            crate::state::breakers::ProviderBreakers::new(map)
        },
        snippet_length: config.conversation_snippet_length,
    };
    // Spawn outbound worker (mock provider)
    let worker_state = state.clone();
    tokio::spawn(async move {
        crate::queue::outbound::run(rx, worker_state).await;
    });

    // Spawn inbound DB worker if pool is available
    if let Some(pool) = db_pool.clone() {
        // Ensure base identities exist (customer id=1, provider id=1) to satisfy FKs for worker inserts
        crate::store_db::seed::seed_identities(&pool).await;
        // Optional: seed demo data to make DB-backed lists non-empty for local runs
        if std::env::var("SEED_DB").ok().as_deref() == Some("1") {
            tokio::spawn({
                let pool = pool.clone();
                async move { crate::store_db::seed::seed_bootstrap(&pool).await }
            });
        }
        let cfg_clone = api_cfg_for_worker.clone();
        tokio::spawn(async move {
            tracing::info!(
                target = "server",
                event = "worker_start",
                worker = "inbound",
                "starting inbound DB worker"
            );
            let w = crate::worker::inbound::InboundWorker::new(pool, cfg_clone);
            w.run().await;
        });
    } else {
        tracing::info!(
            target = "server",
            event = "worker_skip",
            worker = "inbound",
            "no DB pool; inbound worker disabled"
        );
    }

    // TODO (Feature 007): create PgPool and spawn inbound DB worker

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
    // Audit log seeds (US3 T033) using cloned cfg (api_cfg was moved into state)
    if let Some(s) = api_cfg_for_worker
        .provider_sms_seed
        .or(api_cfg_for_worker.provider_seed)
    {
        tracing::info!(target="server", event="provider_seed", provider="sms-mms", seed=%s, "provider seed initialized");
    }
    if let Some(s) = api_cfg_for_worker
        .provider_email_seed
        .or(api_cfg_for_worker.provider_seed)
    {
        tracing::info!(target="server", event="provider_seed", provider="email", seed=%s, "provider seed initialized");
    }

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
    let (queue, rx) = InboundQueue::new(1024);
    let api_cfg = ApiConfig::load();
    crate::providers::common::init_rng_seeds(&api_cfg);
    let brk_thresh = api_cfg.breaker_error_threshold;
    let brk_open_secs = api_cfg.breaker_open_secs;
    // Optionally create DB pool if DATABASE_URL is set
    let db_pool: Option<sqlx::PgPool> = match std::env::var("DATABASE_URL") {
        Ok(url) => match PgPoolOptions::new().max_connections(5).connect(&url).await {
            Ok(pool) => Some(pool),
            Err(e) => {
                tracing::warn!(target="server", event="db_pool_error", error=%e, "failed to create DB pool; continuing without DB");
                None
            }
        },
        Err(_) => None,
    };
    // Build provider registry for graceful-shutdown path
    let provider_registry = {
        use crate::providers::registry::{ChannelKind, Provider, ProviderRegistry};
        use std::sync::Arc;
        let mut reg = ProviderRegistry::new();
        let sms =
            Arc::new(crate::providers::sms_mms::SmsMmsMockProvider::new()) as Arc<dyn Provider>;
        let email =
            Arc::new(crate::providers::email::EmailMockProvider::new()) as Arc<dyn Provider>;
        reg.insert(ChannelKind::Sms, sms.clone());
        reg.insert(ChannelKind::Mms, sms);
        reg.insert(ChannelKind::Email, email);
        reg
    };

    let state = AppState {
        rate: RateLimiter::new(
            api_cfg.rate_limit_per_ip_per_min,
            api_cfg.rate_limit_per_sender_per_min,
        ),
        breaker: CircuitBreaker::new(api_cfg.breaker_error_threshold, api_cfg.breaker_open_secs),
        queue,
        idempotency: IdempotencyStore::new(2 * 60 * 60),
        api: api_cfg.clone(),
        db: db_pool.clone(),
        provider_registry,
        provider_breakers: {
            use crate::middleware::circuit_breaker::CircuitBreaker;
            let mut map = std::collections::HashMap::new();
            map.insert(
                crate::metrics::PROVIDER_LABEL_SMS_MMS.to_string(),
                CircuitBreaker::new(brk_thresh, brk_open_secs),
            );
            map.insert(
                crate::metrics::PROVIDER_LABEL_EMAIL.to_string(),
                CircuitBreaker::new(brk_thresh, brk_open_secs),
            );
            crate::state::breakers::ProviderBreakers::new(map)
        },
        snippet_length: config.conversation_snippet_length,
    };
    // Spawn outbound worker with shutdown signal? For now, fire-and-forget; shutdown will drop rx
    let worker_state = state.clone();
    tokio::spawn(async move {
        crate::queue::outbound::run(rx, worker_state).await;
    });

    // Spawn inbound DB worker if pool is available
    if let Some(pool) = db_pool.clone() {
        // Ensure base identities exist (customer id=1, provider id=1) to satisfy FKs for worker inserts
        crate::store_db::seed::seed_identities(&pool).await;
        // Optional: seed demo data for graceful startup with DB present
        if std::env::var("SEED_DB").ok().as_deref() == Some("1") {
            tokio::spawn({
                let pool = pool.clone();
                async move { crate::store_db::seed::seed_bootstrap(&pool).await }
            });
        }
        let cfg_clone = state.api.clone();
        tokio::spawn(async move {
            tracing::info!(
                target = "server",
                event = "worker_start",
                worker = "inbound",
                "starting inbound DB worker"
            );
            let w = crate::worker::inbound::InboundWorker::new(pool, cfg_clone);
            w.run().await;
        });
    } else {
        tracing::info!(
            target = "server",
            event = "worker_skip",
            worker = "inbound",
            "no DB pool; inbound worker disabled"
        );
    }

    let router = build_router(&config.health_path, state);

    let bind_addr: SocketAddr = ([0, 0, 0, 0], config.port).into();
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|e| format!("failed to bind: {e}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("failed to read local addr: {e}"))?;

    tracing::info!(target: "server", event = "startup", %local_addr, health_path = %config.health_path, "listening");
    if let Some(s) = api_cfg.provider_sms_seed.or(api_cfg.provider_seed) {
        tracing::info!(target="server", event="provider_seed", provider="sms-mms", seed=%s, "provider seed initialized");
    }
    if let Some(s) = api_cfg.provider_email_seed.or(api_cfg.provider_seed) {
        tracing::info!(target="server", event="provider_seed", provider="email", seed=%s, "provider seed initialized");
    }

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
            crate::metrics::record_rate_limited();
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
        crate::metrics::record_breaker_open();
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
