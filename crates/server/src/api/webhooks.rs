use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::errors;
use crate::queue::inbound_events::InboundEvent;
use crate::store_db::inbound_events::insert_inbound_event;
use crate::types::{WebhookEmailRequest, WebhookSmsRequest};

pub(crate) async fn post_sms(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Json(body): Json<WebhookSmsRequest>,
) -> Response {
    if let Some(key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if !state.idempotency.seen_or_insert(key) {
            return (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response();
        }
    }
    // Optional per-sender limit for webhooks as well
    if !state.rate.allow_sender(&body.from) {
        return errors::too_many_requests("Too many requests for sender").into_response();
    }

    if let Some(ref atts) = body.attachments {
        if atts.len() > state.api.max_attachments {
            return errors::bad_request("too many attachments").into_response();
        }
    }

    // If DB is configured, insert inbound_event for worker; otherwise fall back to in-memory queue
    if let Some(pool) = state.db() {
        let channel = body.r#type.to_ascii_lowercase();
        let payload = serde_json::to_value(&body).unwrap_or_else(|_| json!({}));
        let _ = insert_inbound_event(
            &pool,
            &channel,
            &body.from,
            &body.to,
            Some(&body.messaging_provider_id),
            payload,
        )
        .await;
    } else {
        let event = InboundEvent {
            event_name: "webhooks.sms".to_string(),
            payload: serde_json::to_value(&body).unwrap_or_else(|_| json!({})),
            occurred_at: body.timestamp.clone(),
            idempotency_key: None,
            source: "webhook".to_string(),
        };
        let _ = state.queue.enqueue(event).await;
    }

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response()
}

pub(crate) async fn post_email(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Json(body): Json<WebhookEmailRequest>,
) -> Response {
    if let Some(key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if !state.idempotency.seen_or_insert(key) {
            return (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response();
        }
    }
    if let Some(ref atts) = body.attachments {
        if atts.len() > state.api.max_attachments {
            return errors::bad_request("too many attachments").into_response();
        }
    }

    if let Some(pool) = state.db() {
        let payload = serde_json::to_value(&body).unwrap_or_else(|_| json!({}));
        let _ = insert_inbound_event(
            &pool,
            "email",
            &body.from,
            &body.to,
            Some(&body.xillio_id),
            payload,
        )
        .await;
    } else {
        let event = InboundEvent {
            event_name: "webhooks.email".to_string(),
            payload: serde_json::to_value(&body).unwrap_or_else(|_| json!({})),
            occurred_at: body.timestamp.clone(),
            idempotency_key: None,
            source: "webhook".to_string(),
        };
        let _ = state.queue.enqueue(event).await;
    }

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response()
}
