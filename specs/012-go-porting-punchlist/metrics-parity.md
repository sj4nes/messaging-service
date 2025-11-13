# Metrics Parity

This document tracks current metrics fields exposed by the Rust reference server and the expected status for the Go port.

Endpoint: GET /metrics

- Rust: JSON snapshot endpoint used by tests and docs.
- Go: Prometheus text exposition (observed headers: `Content-Type: text/plain; version=0.0.4; ...`). Provide a JSON shim or update docs/tests when consuming.

## Fields (Rust reference)

- rate_limited: total requests rate-limited by IP/sender
- breaker_open: requests short-circuited by global breaker
- dispatch_attempts / dispatch_success / dispatch_rate_limited / dispatch_error: outbound dispatch outcomes
- provider_sms_mms_attempts / provider_sms_mms_success / provider_sms_mms_rate_limited / provider_sms_mms_error: per-provider (sms/mms) counters
- provider_email_attempts / provider_email_success / provider_email_rate_limited / provider_email_error: per-provider (email) counters
- breaker_transitions: global breaker transitions
- provider_sms_mms_breaker_transitions / provider_email_breaker_transitions
- worker_claimed / worker_processed / worker_error / worker_dead_letter: inbound DB worker counters
- worker_latency_avg_us / worker_latency_max_us: inbound worker latency (microseconds)
- invalid_routing: routing errors (Feature 008)
- conversations_created / conversations_reused / conversations_failures: conversation persistence metrics (Feature 009)

## Go Port Status (initial)

- Format: Prometheus text exposition present. Options:
  - Provide a parallel JSON endpoint mirroring these fields for compatibility with JSON-based tooling; or
  - Adapt consumers to parse Prometheus text when hitting the Go service.
- worker_processed: Currently simulated in Go by incrementing within synchronous sms/email handlers. This is provisional; update to real async worker increments or remove simulation before parity closure.
- conversations_*: If Go uses an in-memory fallback initially, expose zeros and document parity plan for DB-backed metrics.

## Parity Guidance

- Counters should be monotonic and reset only on process restart.
- Prefer integer values; avoid floating point.
- If a counter doesn’t apply yet, include it with value 0 and note TODO in Go docs.

