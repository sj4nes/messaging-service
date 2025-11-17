# Research: Go Input Events Queue

This document captures decisions and alternatives evaluated for the input-events queue feature to align the Go server with the Rust reference.

## Decisions

- Processing Model: At-least-once with idempotent persistence
  - Rationale: Simpler operationally vs exactly-once, and current DB schema supports idempotency.
- DLQ/Retry Policy (FR-009): Option B
  - Retry with capped exponential backoff up to 72 hours, max 10 attempts.
  - After exhausting retries or exceeding 72h, move to DLQ; retain DLQ entries for 30 days.
  - Rationale: Bounded operational window, clear SLOs for replay.
- Queue Backend: In-memory channel first
  - Rationale: Keeps scope small, easy to test; a pluggable interface enables future persistent backends (e.g., Postgres, Redis, NATS, SQS).
- Idempotency Key Derivation
  - Key = hash(channel, from, to, normalized sent_at, body hash, customer_id)
  - Rationale: Deterministic and aligns with existing insert logic; enables dedupe on retries.
- Event Schema Versioning
  - Include `schema_version` (e.g., 1). Breaking changes require a new version and dual-consume strategy.

## Alternatives Considered

- Exactly-once semantics with a persistent queue and transactional outbox
  - Rejected for initial scope due to complexity; can evolve towards outbox later.
- Using Postgres as the first queue backend
  - Deferred to keep the first iteration low-risk. Interface allows adding this later.
- Immediate persistence in handlers (status quo)
  - Rejected to restore Rust parity and reduce tail latencies under load.

## Edge Cases

- Empty or whitespace-only body: validation rejects at handler level.
- Missing/invalid timestamps: worker normalizes to server time if absent/invalid.
- Duplicate events (retries or client retries): deduped via idempotency key.
- Long backlog: backpressure via bounded in-memory queue; metrics expose backlog; worker concurrency adjustable.
- DB outage: worker retries with backoff; entries age out to DLQ per policy.

## Observability

- Metrics: enqueue_total, dequeue_total, processed_total, failed_total, retry_total, dlq_total, queue_depth (gauge), worker_active (gauge), processing_latency_histogram.
- Logs: structured with event idempotency key, attempt, backoff, and outcomes; PII redaction for body where required.

## Security

- Maintain existing auth/rate limiting. Avoid logging raw message bodies; log content hashes and metadata instead.
