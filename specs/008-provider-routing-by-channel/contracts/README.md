# Contracts: Provider Routing By Channel

No external API changes: existing endpoints `/api/messages/sms` and `/api/messages/email` remain unchanged.

## Internal Interface (Conceptual)

Provider trait:
- name() -> &'static str
- dispatch(message: OutboundMessage) -> DispatchResult { outcome, error_code?, latency_ms }

DispatchResult outcomes: success | rate_limited | error | timeout.

Registry: channel (sms|mms|email) → Provider.

Breaker: per-provider state machine (closed → open → half_open → closed).

Metrics: counters labeled by provider_name: attempts, success, rate_limited, error, breaker_transitions.

This file documents that no OpenAPI schema modifications are required for this feature.
