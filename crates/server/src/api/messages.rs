use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::json;

use crate::queue::inbound_events::InboundEvent;
use crate::types::{EmailRequest, SmsRequest, Validate};

pub(crate) async fn post_sms(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Json(body): Json<SmsRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(msg) = body.validate(&state.api) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "code": "bad_request", "message": msg })),
        );
    }
    // Idempotency: if key exists and seen already, return 202 without re-enqueueing
    if let Some(key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if !state.idempotency.seen_or_insert(key) {
            return (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" })));
        }
    }
    // Per-sender rate limiting
    if !state.rate.allow_sender(&body.from) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "code": "rate_limited", "message": "Too many requests for sender" })),
        );
    }
    // Basic validation: attachments count
    if let Some(ref atts) = body.attachments {
        if atts.len() > state.api.max_attachments {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "code": "bad_request", "message": "too many attachments" })),
            );
        }
    }

    let event = InboundEvent {
        event_name: "api.messages.sms".to_string(),
        payload: serde_json::to_value(&body).unwrap_or_else(|_| json!({})),
        occurred_at: body.timestamp.clone(),
        idempotency_key: None,
        source: "api".to_string(),
    };
    let _ = state.queue.enqueue(event).await;

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" })))
}

pub(crate) async fn post_email(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Json(body): Json<EmailRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(msg) = body.validate(&state.api) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "code": "bad_request", "message": msg })),
        );
    }
    if let Some(key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if !state.idempotency.seen_or_insert(key) {
            return (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" })));
        }
    }
    if !state.rate.allow_sender(&body.from) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "code": "rate_limited", "message": "Too many requests for sender" })),
        );
    }
    if let Some(ref atts) = body.attachments {
        if atts.len() > state.api.max_attachments {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "code": "bad_request", "message": "too many attachments" })),
            );
        }
    }
    let event = InboundEvent {
        event_name: "api.messages.email".to_string(),
        payload: serde_json::to_value(&body).unwrap_or_else(|_| json!({})),
        occurred_at: body.timestamp.clone(),
        idempotency_key: None,
        source: "api".to_string(),
    };
    let _ = state.queue.enqueue(event).await;

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" })))
}
