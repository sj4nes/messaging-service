# Tasks: API Ingest Endpoints (Feature 005)

Feature Dir: specs/005-api-ingest-endpoints
Spec: specs/005-api-ingest-endpoints/spec.md
Plan: specs/005-api-ingest-endpoints/plan.md

## Phase 1 — Setup

- [X] T001 Create initial server crate structure in crates/server/src/ and crates/server/tests/ (Rust Axum/Tokio) 
- [X] T002 Add Makefile targets for run, test, and lint in Makefile
- [X] T003 Create configuration scaffold (rate limits, body size, breaker) in crates/server/src/config.rs
- [X] T004 Create common error and response schema module in crates/server/src/errors.rs
- [X] T005 Define request validation types (SMS/Email) in crates/server/src/types.rs
- [X] T006 Create inbound_events queue interface and stub in crates/server/src/queue/inbound_events.rs
- [X] T007 Wire basic Axum app/bootstrap in crates/server/src/main.rs

## Phase 2 — Foundational (blocking for all stories)

- [X] T008 Implement JSON content-type enforcement middleware in crates/server/src/middleware/content_type.rs
- [X] T009 Implement Accept/content negotiation helper in crates/server/src/middleware/accept.rs
- [X] T010 Implement request size limits (body size) in crates/server/src/middleware/limits.rs
- [X] T011 Implement idempotency-key extractor/checker in crates/server/src/middleware/idempotency.rs
- [X] T012 Implement per-IP and per-sender rate limiter core in crates/server/src/middleware/rate_limit.rs
- [X] T013 Implement circuit breaker core with states in crates/server/src/middleware/circuit_breaker.rs
- [X] T014 Create OpenAPI contract skeleton in specs/005-api-ingest-endpoints/contracts/openapi.yaml

## Phase 3 — User Story 1 (P1): Ingest messages via POST

Goal: Accept validated POSTs to /api/messages/sms and /api/messages/email, enqueue, return 202.
Independent Test: Use bin/test.sh POSTs; expect 202 and event id in response; verify enqueue.

- [ ] T015 [P] [US1] Define SMS/MMS request schema validation in crates/server/src/types.rs
- [ ] T016 [P] [US1] Define Email request schema validation in crates/server/src/types.rs
- [X] T017 [US1] Implement POST /api/messages/sms handler in crates/server/src/api/messages.rs
- [X] T018 [US1] Implement POST /api/messages/email handler in crates/server/src/api/messages.rs
- [X] T019 [US1] Integrate idempotency behavior in message handlers in crates/server/src/api/messages.rs
- [X] T020 [US1] Apply per-IP and per-sender rate limits to POST message endpoints in crates/server/src/middleware/rate_limit.rs
- [X] T021 [US1] Enqueue inbound_events records from message handlers in crates/server/src/queue/inbound_events.rs
- [X] T022 [US1] Update OpenAPI for message POSTs in specs/005-api-ingest-endpoints/contracts/openapi.yaml

## Phase 4 — User Story 2 (P1): Ingest provider webhooks with safeguards

Goal: Webhook POSTs with per-IP/sender throttling and circuit breakers; enqueue valid events.
Independent Test: Use bin/test.sh webhooks; observe 429 under bursts and 503 during open breaker.

- [X] T023 [US2] Implement POST /api/webhooks/sms handler in crates/server/src/api/webhooks.rs
- [X] T024 [US2] Implement POST /api/webhooks/email handler in crates/server/src/api/webhooks.rs
- [ ] T025 [US2] Apply per-IP and per-sender rate limits to webhook endpoints in crates/server/src/middleware/rate_limit.rs
- [ ] T026 [US2] Integrate circuit breaker checks for webhook routes in crates/server/src/middleware/circuit_breaker.rs
- [ ] T027 [US2] Enqueue inbound_events for webhook handlers in crates/server/src/queue/inbound_events.rs
- [ ] T028 [US2] Update OpenAPI for webhook POSTs in specs/005-api-ingest-endpoints/contracts/openapi.yaml

## Phase 5 — User Story 3 (P2): Retrieve conversations

Goal: GET /api/conversations with content negotiation and pagination.
Independent Test: GET with Accept header; verify JSON response and paging metadata.

- [ ] T029 [P] [US3] Define list/paging DTOs for conversations in crates/server/src/types.rs
- [ ] T030 [US3] Implement GET /api/conversations handler in crates/server/src/api/conversations.rs
- [ ] T031 [US3] Implement content negotiation for GET conversations in crates/server/src/middleware/accept.rs
- [ ] T032 [US3] Update OpenAPI for conversations GET in specs/005-api-ingest-endpoints/contracts/openapi.yaml

## Phase 6 — User Story 4 (P2): Retrieve messages for a conversation

Goal: GET /api/conversations/{id}/messages with content negotiation and pagination.
Independent Test: GET messages by id; verify 200 and paging; 404 on unknown id.

- [ ] T033 [P] [US4] Define list/paging DTOs for messages in crates/server/src/types.rs
- [ ] T034 [US4] Implement GET /api/conversations/{id}/messages handler in crates/server/src/api/conversations.rs
- [ ] T035 [US4] Implement content negotiation for messages in crates/server/src/middleware/accept.rs
- [ ] T036 [US4] Update OpenAPI for conversation messages GET in specs/005-api-ingest-endpoints/contracts/openapi.yaml

## Final Phase — Polish & Cross-Cutting

- [ ] T037 Add consistent error responses (code, message, details) in crates/server/src/errors.rs
- [ ] T038 Add structured logging with correlation IDs and redaction in crates/server/src/observability/logging.rs
- [ ] T039 Expose minimal metrics (counts, rates, throttles, breaker state) in crates/server/src/observability/metrics.rs
- [ ] T040 Add configuration for limits and breaker windows in config files (e.g., crates/server/config/default.toml)
- [ ] T041 Add Makefile targets for running server plus watch/test in Makefile
- [ ] T042 Wire task to update agent context after design in .specify/scripts/bash/update-agent-context.sh

## Dependencies

- Story order: US1 (P1) → US2 (P1) → US3 (P2) → US4 (P2)
- Foundational middleware (limits, accept, idempotency, rate limit, breaker) must precede US1/US2 handlers.

## Parallelization Examples

- US1: T015/T016 can be implemented in parallel; T017/T018 can proceed after validation types.
- US2: T023/T024 can be worked in parallel; T025/T026 independent of each other, integrate before T027.
- US3/US4: DTO definitions (T029/T033) can be parallel; content negotiation (T031/T035) parallel to handlers once DTOs are ready.

## Implementation Strategy (MVP first)

- MVP: Complete Phases 1–3 (Setup, Foundational, US1) to accept and enqueue messages with proper validation and limits.
- Incremental: Add US2 webhooks safeguards; then read-side US3/US4; finalize with polish & observability.
