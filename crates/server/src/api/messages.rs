use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::errors;
use crate::queue::inbound_events::InboundEvent;
use crate::store::messages as message_store;
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

    // Persist outbound: always in-memory for conversation listing fallback; additionally into DB if available
    let msg_id = if body.r#type.eq_ignore_ascii_case("mms") {
        message_store::insert_outbound_mms(
            &body.from,
            &body.to,
            &body.body,
            &body.attachments,
            &body.timestamp,
        )
    } else {
        message_store::insert_outbound_sms(
            &body.from,
            &body.to,
            &body.body,
            &body.attachments,
            &body.timestamp,
        )
    };
    if let Some(pool) = state.db() {
        // Ensure identities and test mapping exist after potential DB reset without server restart
        crate::store_db::seed::seed_minimum_if_needed(&pool).await;
        // Best-effort DB persistence; ignore errors to keep API responsive
        let channel = if body.r#type.eq_ignore_ascii_case("mms") {
            "mms"
        } else {
            "sms"
        };
        if let Err(e) = crate::store_db::messages::insert_outbound(
            &pool,
            channel,
            &body.from,
            &body.to,
            &body.body,
            &body.attachments.clone().unwrap_or_default(),
            &body.timestamp,
        )
        .await
        {
            tracing::warn!(target="server", event="db_outbound_persist_fail", error=%e, channel=%channel, "failed to persist outbound message to DB");
        }
    }
    let mut payload = serde_json::to_value(&body).unwrap_or_else(|_| json!({}));
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("message_id".to_string(), json!(msg_id));
    }
    let event = InboundEvent {
        event_name: "api.messages.sms".to_string(),
        payload,
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
    // Persist outbound email: in-memory + DB if available
    let msg_id = message_store::insert_outbound_email(
        &body.from,
        &body.to,
        &body.body,
        &body.attachments,
        &body.timestamp,
    );
    if let Some(pool) = state.db() {
        crate::store_db::seed::seed_minimum_if_needed(&pool).await;
        if let Err(e) = crate::store_db::messages::insert_outbound(
            &pool,
            "email",
            &body.from,
            &body.to,
            &body.body,
            &body.attachments.clone().unwrap_or_default(),
            &body.timestamp,
        )
        .await
        {
            tracing::warn!(target="server", event="db_outbound_email_persist_fail", error=%e, "failed to persist outbound email to DB");
        }
    }
    let mut payload = serde_json::to_value(&body).unwrap_or_else(|_| json!({}));
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("message_id".to_string(), json!(msg_id));
    }
    let event = InboundEvent {
        event_name: "api.messages.email".to_string(),
        payload,
        occurred_at: body.timestamp.clone(),
        idempotency_key: None,
        source: "api".to_string(),
    };
    let _ = state.queue.enqueue(event).await;

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response()
}
