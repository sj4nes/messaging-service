use tokio::sync::mpsc::Receiver;

use crate::middleware::circuit_breaker::BreakerState;
use crate::providers::mock::{pick_outcome, Outcome};
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

        info!(target="server", event="mock_dispatch_attempt", mock=true, event_name=%evt.event_name, "processing outbound event with provider mock");

        // Short-circuit if breaker is open
        if state.breaker.before_request() == BreakerState::Open {
            crate::metrics::record_breaker_open();
            info!(
                target = "server",
                event = "mock_dispatch_short_circuit",
                mock = true,
                breaker_state = "open",
                "breaker open; short-circuiting dispatch"
            );
            continue;
        }

        crate::metrics::record_dispatch_attempt();
        let outcome = pick_outcome(&state.api);
        match outcome {
            Outcome::Success => {
                crate::metrics::record_dispatch_success();
                // Successful attempt closes/keeps closed breaker
                state.breaker.record_success();
                info!(
                    target = "server",
                    event = "mock_dispatch_outcome",
                    mock = true,
                    outcome = "success",
                    "mock provider dispatch succeeded"
                );
            }
            Outcome::RateLimited => {
                crate::metrics::record_dispatch_rate_limited();
                // No breaker change on 429
                info!(
                    target = "server",
                    event = "mock_dispatch_outcome",
                    mock = true,
                    outcome = "rate_limited",
                    "mock provider returned 429 rate limit"
                );
            }
            Outcome::Error | Outcome::Timeout => {
                crate::metrics::record_dispatch_error();
                let before = state.breaker.state();
                state.breaker.record_failure();
                let after = state.breaker.state();
                if before != after {
                    crate::metrics::record_breaker_transition();
                    info!(target="server", event="mock_breaker_transition", mock=true, from=?before, to=?after, "circuit breaker state transitioned");
                }
                let label = if matches!(outcome, Outcome::Timeout) {
                    "timeout"
                } else {
                    "error"
                };
                info!(target="server", event="mock_dispatch_outcome", mock=true, outcome=%label, "mock provider dispatch failed");
            }
        }
    }
}
