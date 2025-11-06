use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::errors;
use crate::queue::inbound_events::InboundEvent;
use crate::types::{EmailRequest, SmsRequest, Validate};

pub(crate) async fn post_sms(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Json(body): Json<SmsRequest>,
) -> Response {
    if let Err(msg) = body.validate(&state.api) {
        return errors::bad_request(msg).into_response();
    }
    // Idempotency: if key exists and seen already, return 202 without re-enqueueing
    if let Some(key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if !state.idempotency.seen_or_insert(key) {
            return (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response();
        }
    }
    // Per-sender rate limiting
    if !state.rate.allow_sender(&body.from) {
        return errors::too_many_requests("Too many requests for sender").into_response();
    }
    // Basic validation: attachments count
    if let Some(ref atts) = body.attachments {
        if atts.len() > state.api.max_attachments {
            return errors::bad_request("too many attachments").into_response();
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

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response()
}

pub(crate) async fn post_email(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Json(body): Json<EmailRequest>,
) -> Response {
    if let Err(msg) = body.validate(&state.api) {
        return errors::bad_request(msg).into_response();
    }
    if let Some(key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if !state.idempotency.seen_or_insert(key) {
            return (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response();
        }
    }
    if !state.rate.allow_sender(&body.from) {
        return errors::too_many_requests("Too many requests for sender").into_response();
    }
    if let Some(ref atts) = body.attachments {
        if atts.len() > state.api.max_attachments {
            return errors::bad_request("too many attachments").into_response();
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

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response()
}
