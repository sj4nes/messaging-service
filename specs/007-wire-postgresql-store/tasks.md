# Tasks: Wire PostgreSQL Store

Feature Dir: /Users/sjanes/work2/hatch/messaging-service/specs/007-wire-postgresql-store
Plan: plan.md | Spec: spec.md | Research: research.md | Data Model: data-model.md | Contracts: contracts/openapi.yaml | Quickstart: quickstart.md

## Phase 1 – Setup

- [x] T001 Ensure migrations folder exists at crates/db-migrate/migrations_sqlx/
- [x] T002 Create migration (UP): alter inbound_events to add channel, from, to, provider_message_id, processor_id, error_code, error_message, updated_at, processed_at; add UNIQUE(channel, provider_message_id) WHERE provider_message_id IS NOT NULL at crates/db-migrate/migrations_sqlx/0007_alter_inbound_events_unified.up.sql
- [x] T003 Create migration (DOWN): drop added columns and unique index at crates/db-migrate/migrations_sqlx/0007_alter_inbound_events_unified.down.sql
- [x] T004 Add config knobs in crates/server/src/config.rs (batch_size, claim_timeout_secs, max_retries, backoff_base_ms)
- [x] T005 [P] Extend metrics in crates/server/src/metrics.rs for worker_queued, worker_claimed, worker_processed, worker_error, worker_dead_letter, and latency

## Phase 2 – Foundational

- [x] T010 Create DB module root crates/server/src/store_db/mod.rs
- [x] T011 Create inbound events store crates/server/src/store_db/inbound_events.rs (insert, claim_batch, mark_processed, mark_error, reap_stale)
- [x] T012 Create messages store crates/server/src/store_db/messages.rs (insert_from_inbound, helpers)
- [x] T013 Create conversations store crates/server/src/store_db/conversations.rs (upsert_on_message, list, list_messages)
- [x] T014 [P] Add shared normalization helper crates/server/src/store_db/normalize.rs (channel-aware address normalization and conversation key)

## Phase 3 – [US1] Persist inbound events (P1)

- [x] T015 [US1] Implement insert_inbound_event in crates/server/src/store_db/inbound_events.rs using SQLx and idempotent upsert on (channel, provider_message_id)
- [x] T016 [US1] Wire provider webhooks to DB: update crates/server/src/api/provider_mock.rs to persist inbound_events instead of in-memory
- [x] T017 [US1] Wire real webhooks to DB: update crates/server/src/api/webhooks.rs to persist inbound_events
- [x] T018 [P] [US1] Add tracing (mock=true, event ids) in provider inbound handlers crates/server/src/api/provider_mock.rs
- [x] T019 [US1] Verify duplicate inbound (same provider id) returns 202 without duplicate rows (graceful unique violation handling)

## Phase 4 – [US2] Background worker processes events (P1)

- [x] T020 [US2] Add worker module crates/server/src/worker/inbound.rs to claim with SELECT … FOR UPDATE SKIP LOCKED and set status=processing, processor_id
- [x] T021 [US2] Implement processing path: create Message rows and mark inbound_events processed in crates/server/src/worker/inbound.rs
- [x] T022 [P] [US2] Implement retry policy with exponential backoff (attempt_count, next_attempt_at) and mark dead_letter after max_retries in crates/server/src/worker/inbound.rs
- [x] T023 [US2] Add metrics increments and timings in worker using crates/server/src/metrics.rs
- [x] T024 [US2] Wire worker startup in crates/server/src/lib.rs (spawn background task; read config knobs)

## Phase 5 – [US3] Conversations read from DB (P2)

- [x] T025 [US3] Implement conversations list using DB: crates/server/src/store_db/conversations.rs (aggregate by key, order by last_activity_at desc)
- [x] T026 [US3] Implement messages list using DB: crates/server/src/store_db/conversations.rs (order by timestamp asc, pagination)
- [x] T027 [P] [US3] Update API handlers to use DB stores: crates/server/src/api/conversations.rs (list_conversations, list_messages)

## Phase 6 – Polish & Cross-cutting

- [x] T028 Document DB quickstart and verify steps in specs/007-wire-postgresql-store/quickstart.md (psql queries, config)
- [x] T029 Update README anchors to reference this feature’s quickstart (optional) at README.md
- [x] T030 [P] Update tests to assert conversations non-empty after activity: tests/http/tests.json (add "assert": ".items | length > 0" to the List conversations case)
- [ ] T031 Ensure SQLX_OFFLINE compatibility for CI builds (sqlx-data.json refresh if needed)
- [x] T032 Add basic health/metrics note for worker throughput in README.md

## Dependencies

- Phase 1 (Setup) → Phase 2 (Foundational) → US1 and US2
- US2 depends on US1’s persistence and foundational stores
- US3 depends on message persistence from US2 (or outbound path already persisted) and DB conversation store

Order:
1) Setup (T001–T009)
2) Foundational (T010–T014)
3) US1 (T015–T019)
4) US2 (T020–T024)
5) US3 (T025–T027)
6) Polish (T028–T032)

## Parallel Opportunities

- [P] T009 metrics, [P] T014 normalization helper can proceed in parallel with other foundational tasks
- [P] T018 tracing can proceed while wiring inserts
- [P] T022 retry/backoff logic can be developed alongside T020–T021 worker basics
- [P] T027 API handler swap to DB for conversations after store functions exist
- [P] T030 tests.json asserts can be added any time after conversations API works

## Independent Test Criteria

- US1: Post an inbound webhook; verify inbound_events row with status="received"; duplicate post does not create a new row; restart retains data
- US2: Insert a received inbound_event row and run server; worker claims, creates a Message row, and sets status to processed; on forced error, status=error and attempt_count increments until dead_letter
- US3: GET /api/conversations returns items > 0 after events; GET /api/conversations/{id}/messages returns a sorted list with correct pagination metadata

## Suggested MVP Scope

- Deliver US1 (persist inbound events) and basic worker path (US2 minimal: claim + processed) in first increment; DB reads for conversations can follow

## Format Validation

All tasks follow required checklist format:
- Checkbox prefix
- Sequential Task IDs (T001…)
- [P] marker only on parallelizable tasks
- [USx] marker only on user story phases
- Descriptions include exact file paths
