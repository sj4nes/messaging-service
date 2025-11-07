use tokio::sync::mpsc::Receiver;

use crate::middleware::circuit_breaker::BreakerState;
use crate::providers::mock::{pick_outcome, Outcome};
use crate::queue::inbound_events::InboundEvent;

/// Run the outbound worker consuming events and simulating provider dispatch.
pub(crate) async fn run(mut rx: Receiver<InboundEvent>, state: crate::AppState) {
	while let Some(evt) = rx.recv().await {
		// Only process outbound api events; skip others for now
		let is_outbound = matches!(evt.event_name.as_str(), "api.messages.sms" | "api.messages.email");
		if !is_outbound {
			continue;
		}

		// Short-circuit if breaker is open
		if state.breaker.before_request() == BreakerState::Open {
			crate::metrics::record_breaker_open();
			continue;
		}

		crate::metrics::record_dispatch_attempt();
		match pick_outcome(&state.api) {
			Outcome::Success => {
				crate::metrics::record_dispatch_success();
				// Successful attempt closes/keeps closed breaker
				state.breaker.record_success();
			}
			Outcome::RateLimited => {
				crate::metrics::record_dispatch_rate_limited();
				// No breaker change on 429
			}
			Outcome::Error | Outcome::Timeout => {
				crate::metrics::record_dispatch_error();
				let before = state.breaker.state();
				state.breaker.record_failure();
				let after = state.breaker.state();
				if before != after {
					crate::metrics::record_breaker_transition();
				}
			}
		}
	}
}
