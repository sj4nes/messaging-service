# Metrics Parity

This document tracks current metrics fields exposed by the Rust reference server and the expected status for the Go port.

Endpoint: GET /metrics (JSON)

- Note: The test runner polls /metrics and attempts to parse JSON. If the Go port uses Prometheus text format, document that and provide a separate JSON shim or adjust the test harness accordingly.

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

- Format: If Go uses Prometheus client, either:
  - provide a JSON endpoint mirroring these fields for tests, or
  - adapt the test harness to parse text exposition and extract counters.
- worker_processed: If no inbound DB worker is implemented yet in Go, keep counter at 0 and document absence. Don’t simulate increments to avoid misleading observability.
- conversations_*: If Go uses an in-memory fallback initially, expose zeros and document parity plan for DB-backed metrics.

## Parity Guidance

- Counters should be monotonic and reset only on process restart.
- Prefer integer values; avoid floating point.
- If a counter doesn’t apply yet, include it with value 0 and note TODO in Go docs.

