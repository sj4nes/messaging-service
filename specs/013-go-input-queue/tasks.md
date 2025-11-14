# Tasks: Go Input Events Queue for Messaging Server (013-go-input-queue)

This plan is organized per speckit.tasks rules. Tasks are grouped by phases and user stories, each independently testable. All tasks follow the strict checklist format.

## Phase 1: Setup

- [X] T001 Create queue package directories in go/internal/queue and go/internal/queue/memory
- [X] T002 Create worker package directory in go/internal/worker
- [X] T003 Ensure contracts exist for endpoints in specs/013-go-input-queue/contracts/openapi.yaml
- [X] T004 Add developer notes in specs/013-go-input-queue/quickstart.md for in-memory queue mode
- [ ] T005 Add Makefile targets (optional) for running server+worker if needed in Makefile

## Phase 2: Foundational

- [X] T006 Implement queue interface in go/internal/queue/queue.go (Publish, Subscribe/Receive, Ack, Nack)
- [X] T007 [P] Implement in-memory queue backend in go/internal/queue/memory/memory.go (bounded buffer, blocking semantics)
- [X] T008 Add event DTO definition (OutboundMessageEvent) in go/internal/queue/queue.go with schema_version=1
- [X] T009 Add worker skeleton in go/internal/worker/worker.go (constructor, Start/Stop, handler func injection)
 - [X] T010 [P] Add metrics counters/gauges for enqueue/dequeue/processed/failed/backlog in go/internal/metrics/queue.go (or extend existing metrics package)
- [ ] T011 [P] Add HTTP test utilities to seed customers and reset DB in go/api/testutil_db.go
 - [X] T012 Reintroduce or expose repository method for idempotent insert in go/internal/db/repository/messages_repository.go (InsertOutbound)
 - [X] T013 [P] Ensure DB-backed store has read methods and optionally thin write wrappers in go/internal/db/store/store.go

## Phase 3: User Story 1 (P1) — Enqueue valid outbound requests

Goal: Handlers validate and enqueue; return 202. Independent tests use in-memory queue.

 - [X] T014 [US1] Add handler contract tests for 202/4xx using in-memory queue in go/api/messages_test.go
 - [X] T015 [P] [US1] Wire queue producer into SMS handler in go/api/messages.go (replace direct persistence)
 - [X] T016 [P] [US1] Wire queue producer into Email handler in go/api/messages.go (replace direct persistence)
 - [X] T017 [US1] On success, return {"status":"accepted"} (and optional idempotencyKey) in go/api/messages.go
 - [X] T018 [US1] Validate inputs (non-empty body, valid addresses) and ensure no enqueue on 4xx in go/api/messages.go
 - [X] T019 [US1] Add metric increments for enqueue attempts/success/failure in go/api/messages.go

Independent Test Criteria (US1):
- POST valid SMS/Email → 202 + exactly one event in queue.
- POST invalid request → 4xx + zero events in queue.

## Phase 4: User Story 2 (P2) — Worker processes queued events

Goal: Worker consumes events, performs durable persistence, idempotent, updates conversation counters.

- [X] T020 [US2] Implement event-to-persistence mapping in go/internal/worker/worker.go using repository.InsertOutbound
- [ ] T021 [P] [US2] Implement idempotency key derivation (if absent) and timestamp normalization in go/internal/worker/worker.go
- [ ] T022 [P] [US2] Add retry with capped exponential backoff and attempt tracking in go/internal/worker/worker.go
- [ ] T023 [US2] Implement DLQ behavior (in-memory) and retention hooks in go/internal/worker/worker.go
- [X] T024 [P] [US2] Add worker integration tests that seed events and assert DB state in go/internal/worker/worker_test.go
- [X] T025 [US2] Ensure duplicate events (same idempotency key) do not create duplicates in go/internal/worker/worker_test.go

Independent Test Criteria (US2):
- Given a valid event, one message + conversation persisted; reprocessing same event is idempotent.

## Phase 5: User Story 3 (P3) — Configuration and visibility

Goal: Configurable queue mode, concurrency, retry windows; metrics/logging for operability.

- [ ] T026 [US3] Add configuration parsing for QUEUE_BACKEND, WORKER_ENABLED, WORKER_CONCURRENCY, RETRY_MAX_ATTEMPTS, RETRY_MAX_AGE, DLQ_RETENTION in go/cmd/server/main.go and go/internal/config/config.go
- [ ] T027 [P] [US3] Expose queue/worker metrics at metrics endpoint; document names in specs/013-go-input-queue/quickstart.md
- [ ] T028 [P] [US3] Add structured logs with idempotency key, attempt, outcome, and redacted body in go/internal/worker/worker.go
- [ ] T029 [US3] Add startup wiring: in-process worker when WORKER_ENABLED=true in go/cmd/server/main.go
- [ ] T030 [P] [US3] Add smoke tests to verify metrics/logging presence in go/tests/metrics_logging_test.go

Independent Test Criteria (US3):
- Metrics show enqueue/processing rates and backlog; failures logged with context; toggling config changes behavior.

## Final Phase: Polish & Cross-Cutting

- [ ] T031 Update docs: specs/013-go-input-queue/quickstart.md with run instructions and troubleshooting
- [ ] T032 Add README section linking the new feature in README.md
- [ ] T033 Ensure code lint/format pass for Go and update Makefile if necessary
- [ ] T034 Add CI step (if applicable) for worker tests in .github/workflows/ci.yml

## Dependencies

- Phase 1 → Phase 2 → US1 (P1) → US2 (P2) → US3 (P3) → Polish
- US1 does not strictly block US2 (worker can be tested by seeding queue), but end-to-end relies on US1 enqueue

## Parallel Execution Examples

- T007, T010, T011, T013 can proceed in parallel after T006 (interfaces in place)
- US1 handlers (T015, T016) can be done in parallel once queue producer contract (T006/T008) exists
- US2 retry/DLQ (T022, T023) can be parallelized after base worker loop (T020)

## Implementation Strategy

- MVP Scope: Complete US1 (P1) with in-memory queue and handler enqueue (T014–T019)
- Incrementally add US2 worker processing next, then US3 operability

