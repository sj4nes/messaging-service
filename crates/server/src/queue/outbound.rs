use tokio::sync::mpsc::Receiver;

use crate::middleware::circuit_breaker::BreakerState;
use crate::providers::mock::Outcome;
use crate::providers::registry::{ChannelKind, OutboundMessage};
use crate::queue::inbound_events::InboundEvent;
use tracing::info;

/// Run the outbound worker consuming events and simulating provider dispatch.
pub(crate) async fn run(mut rx: Receiver<InboundEvent>, state: crate::AppState) {
    while let Some(evt) = rx.recv().await {
        // Only process outbound api events; skip others for now
        let is_outbound = matches!(
            evt.event_name.as_str(),
            "api.messages.sms" | "api.messages.email"
        );
        if !is_outbound {
            continue;
        }

        // Determine channel kind based on event name
        let channel = match evt.event_name.as_str() {
            "api.messages.sms" => {
                // differentiate SMS vs MMS using payload type field if present
                let kind = evt
                    .payload
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("sms");
                if kind.eq_ignore_ascii_case("mms") {
                    ChannelKind::Mms
                } else {
                    ChannelKind::Sms
                }
            }
            "api.messages.email" => ChannelKind::Email,
            _ => continue,
        };

        // Provider lookup
        let provider = match state.provider_registry.get(channel) {
            Some(p) => p.clone(),
            None => {
                crate::metrics::record_invalid_routing();
                info!(target="server", event="provider_missing", channel=?channel, "no provider registered for channel");
                continue;
            }
        };

        info!(target="server", event="dispatch_attempt", provider=%provider.name(), channel=%channel.as_str(), event_name=%evt.event_name, "processing outbound event");
        crate::metrics::record_provider_attempt(provider.name());

        // Use per-provider breaker (fallback to global if not found)
        // Per-provider breaker lookup
        let provider_breaker = state
            .provider_breakers
            .get(provider.name())
            .unwrap_or(&state.breaker);
        if provider_breaker.before_request() == BreakerState::Open {
            crate::metrics::record_breaker_open();
            info!(
                target = "server",
                event = "dispatch_short_circuit",
                provider = %provider.name(),
                breaker_state = "open",
                "provider breaker open; short-circuiting dispatch"
            );
            continue;
        }

        crate::metrics::record_dispatch_attempt();
        // Build outbound message (subset fields used currently)
        let outbound = OutboundMessage {
            channel,
            to: evt
                .payload
                .get("to")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            from: evt
                .payload
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            body: evt
                .payload
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            attachments: evt
                .payload
                .get("attachments")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            idempotency_key: evt.idempotency_key.clone(),
        };

        // Tag provider on stored outbound message if id present
        if let Some(msg_id) = evt.payload.get("message_id").and_then(|v| v.as_str()) {
            let _ = crate::store::messages::set_outbound_provider(msg_id, provider.name());
        }

        // Execute provider dispatch (mock)
        let result = provider.dispatch(&outbound, &state.api);
        let outcome = result.outcome;
        match outcome {
            Outcome::Success => {
                crate::metrics::record_dispatch_success();
                crate::metrics::record_provider_success(provider.name());
                // Successful attempt may transition breaker (e.g., half-open -> closed)
                let before = provider_breaker.state();
                provider_breaker.record_success();
                let after = provider_breaker.state();
                if before != after {
                    crate::metrics::record_breaker_transition();
                    crate::metrics::record_provider_breaker_transition(provider.name());
                    info!(target = "server", event = "breaker_transition", provider=%provider.name(), from=?before, to=?after, "circuit breaker state transitioned");
                }
                info!(target = "server", event = "dispatch_outcome", provider=%provider.name(), outcome="success", channel=%channel.as_str(), "provider dispatch succeeded");
            }
            Outcome::RateLimited => {
                crate::metrics::record_dispatch_rate_limited();
                crate::metrics::record_provider_rate_limited(provider.name());
                // No breaker change on 429
                info!(target = "server", event = "dispatch_outcome", provider=%provider.name(), outcome="rate_limited", channel=%channel.as_str(), "provider returned 429 rate limit");
            }
            Outcome::Error | Outcome::Timeout => {
                crate::metrics::record_dispatch_error();
                crate::metrics::record_provider_error(provider.name());
                // Record failure against provider-specific breaker (fallback may be global)
                let before = provider_breaker.state();
                provider_breaker.record_failure();
                let after = provider_breaker.state();
                if before != after {
                    // Global transition counter retained + per-provider counter
                    crate::metrics::record_breaker_transition();
                    crate::metrics::record_provider_breaker_transition(provider.name());
                    info!(target="server", event="breaker_transition", provider=%provider.name(), from=?before, to=?after, "circuit breaker state transitioned");
                }
                let label = if matches!(outcome, Outcome::Timeout) {
                    "timeout"
                } else {
                    "error"
                };
                info!(target="server", event="dispatch_outcome", provider=%provider.name(), outcome=%label, channel=%channel.as_str(), "provider dispatch failed");
            }
        }
    }
}
