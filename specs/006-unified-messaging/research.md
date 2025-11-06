# Research and Decisions: Unified Messaging

Date: 2025-11-06
Branch: 006-unified-messaging

## Decisions

1. Provider Mocks In-Process
- Decision: Implement provider mocks (SMS/MMS/Email) as in-process components with configurable probabilities (success %, 429 %, 5xx %). Seedable RNG for deterministic tests.
- Rationale: Simplifies orchestration and keeps IO minimal while exercising resilience logic.
- Alternatives: Separate services with HTTP; rejected for complexity and non-essential network flakiness.

2. Queue Consumption Model
- Decision: Reuse existing `InboundQueue` and spawn background workers (Tokio tasks) to process outbound events and simulate inbound generation.
- Rationale: Fits current architecture; avoids introducing new message broker for this feature.
- Alternatives: External broker (e.g., NATS/Kafka) rejected due to scope.

3. Circuit Breaker and Retries
- Decision: Use existing breaker; on 5xx outcomes increment failures; on 429 treat as retryable with backoff (no exponential policy mandated for this feature). Half-open requires a successful trial before closing.
- Rationale: Validates resilience; aligns with previous infrastructure.
- Alternatives: Pluggable backoff strategies; deferred.

4. Conversation Grouping (Stretch)
- Decision: Group by (channel, normalized_from, normalized_to); direction-agnostic. Use normalized phone/email formats; if normalization fails, fallback to raw strings.
- Rationale: Clear deterministic grouping; independent of persistence implementation.
- Alternatives: Thread-based or time-window grouping; rejected as overfit.

5. Metrics and Observability
- Decision: Extend metrics to include dispatch_attempts, dispatch_success, dispatch_rate_limited, dispatch_error, breaker_transitions.
- Rationale: Visibility into success/failure mix and breaker behavior.
- Alternatives: Prometheus exporter; deferred.

## Clarifications Resolved

- Data persistence: Not required to expand beyond current scope; in-memory acceptable for feature validation. If DB present, can wire persistence without changing API.
- Failure profiles: Configurable via file/env. Global per-channel profiles suffice for scope.
- Determinism: Enable seed via env variable to make tests reproducible.

## Open Alternatives (Deferred)

- External mock-provider processes communicating over HTTP.
- Sophisticated retry/backoff and DLQ semantics.
- Prometheus metrics/OTel tracing integration.

