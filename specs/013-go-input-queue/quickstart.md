# Quickstart: Go Input Events Queue

This guide shows how to run the HTTP server and the background worker with the in-memory queue.

## Prerequisites

- Go 1.22+
- PostgreSQL running with schema migrated (matches Rust SQLx migrations)
- Environment variable `DATABASE_URL` set for tests/server

## Configuration (environment variables)

- QUEUE_BACKEND=memory (default)
- WORKER_ENABLED=true | false (default true in dev)
- WORKER_CONCURRENCY=4 (default 2)
- RETRY_MAX_ATTEMPTS=10 (per spec Option B)
- RETRY_MAX_AGE=72h (per spec Option B)
- DLQ_RETENTION=720h (30d)

## Run

- In one process (server runs worker goroutine):
  - Start the server binary; it initializes the in-memory queue and worker if WORKER_ENABLED=true.
- In two processes:
  - Start server (producer only, WORKER_ENABLED=false)
  - Start worker binary (consumer, connects to same queue backend when using a persistent backend; for in-memory, run in-process).

## Observability

- Metrics: Exposed at the server's metrics endpoint (Prometheus). Key metrics include:
  - messaging_queue_enqueue_total, dequeue_total, processed_total, failed_total, retry_total, dlq_total
  - messaging_queue_depth, messaging_worker_active
  - messaging_processing_latency_seconds_bucket
- Logs: Structured with idempotency key, attempt, outcome. Bodies are not logged (only hashes).

## Notes

- For local dev with in-memory queue, the worker must run in the same process as the server.
- For integration tests, seed a `customers` row to satisfy message FKs.
