use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::sync::{OnceLock, RwLock};

use crate::errors;
use crate::queue::inbound_events::InboundEvent;
use crate::types::{ProviderInboundRequest, Validate};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderMockConfig {
    pub timeout_pct: u32,
    pub error_pct: u32,
    pub ratelimit_pct: u32,
    pub seed: Option<u64>,
}

impl ProviderMockConfig {
    fn from_api(cfg: &crate::config::ApiConfig) -> Self {
        Self {
            timeout_pct: cfg.provider_timeout_pct,
            error_pct: cfg.provider_error_pct,
            ratelimit_pct: cfg.provider_ratelimit_pct,
            seed: cfg.provider_seed,
        }
    }
}

fn global_config() -> &'static RwLock<ProviderMockConfig> {
    static CELL: OnceLock<RwLock<ProviderMockConfig>> = OnceLock::new();
    CELL.get_or_init(|| {
        RwLock::new(ProviderMockConfig {
            timeout_pct: 0,
            error_pct: 0,
            ratelimit_pct: 0,
            seed: None,
        })
    })
}

/// POST /api/provider/mock/inbound
/// Accept provider-originated inbound message events and enqueue for processing.
pub(crate) async fn post_inbound(
    State(state): State<crate::AppState>,
    Json(body): Json<ProviderInboundRequest>,
) -> axum::response::Response {
    // Validate basic shape per variant
    let valid = match &body {
        ProviderInboundRequest::Sms(s) | ProviderInboundRequest::Mms(s) => s.validate(&state.api),
        ProviderInboundRequest::Email(e) => e.validate(&state.api),
    };
    if let Err(msg) = valid {
        return errors::bad_request(msg).into_response();
    }

    // Normalize event name by variant for downstream handlers
    let (event_name, occurred_at) = match &body {
        ProviderInboundRequest::Sms(s) => ("provider.mock.sms.inbound", s.timestamp.clone()),
        ProviderInboundRequest::Mms(s) => ("provider.mock.mms.inbound", s.timestamp.clone()),
        ProviderInboundRequest::Email(e) => ("provider.mock.email.inbound", e.timestamp.clone()),
    };

    let event = InboundEvent {
        event_name: event_name.to_string(),
        payload: serde_json::to_value(&body).unwrap_or_else(|_| json!({})),
        occurred_at,
        idempotency_key: None,
        source: "provider.mock".to_string(),
    };
    let _ = state.queue.enqueue(event).await;

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response()
}

/// GET /api/provider/mock/config
pub(crate) async fn get_config(State(state): State<crate::AppState>) -> axum::response::Response {
    // Initialize from ApiConfig only if global is still defaults
    let lock = global_config();
    {
        let current = lock.read().unwrap();
        // If all zeros and None, seed from ApiConfig
        if current.timeout_pct == 0
            && current.error_pct == 0
            && current.ratelimit_pct == 0
            && current.seed.is_none()
        {
            drop(current);
            let mut w = lock.write().unwrap();
            *w = ProviderMockConfig::from_api(&state.api);
        }
    }
    let cfg = lock.read().unwrap().clone();
    Json(cfg).into_response()
}

/// PUT /api/provider/mock/config
pub(crate) async fn put_config(Json(body): Json<ProviderMockConfig>) -> axum::response::Response {
    let lock = global_config();
    {
        let mut w = lock.write().unwrap();
        *w = body.clone();
    }
    Json(body).into_response()
}
