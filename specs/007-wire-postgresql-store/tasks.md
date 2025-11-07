# Tasks: Wire PostgreSQL Store

Feature Dir: /Users/sjanes/work2/hatch/messaging-service/specs/007-wire-postgresql-store
Plan: plan.md | Spec: spec.md | Research: research.md | Data Model: data-model.md | Contracts: contracts/openapi.yaml | Quickstart: quickstart.md

## Phase 1 – Setup

- [ ] T001 Ensure migrations folder exists at migrations_sqlx/ (verify in repo root)
- [ ] T002 Create migration: inbound_events table at migrations_sqlx/20251106_01_wire_inbound_events_up.sql
- [ ] T003 Create migration (down): inbound_events table at migrations_sqlx/20251106_01_wire_inbound_events_down.sql
- [ ] T004 Create migration: messages table at migrations_sqlx/20251106_02_wire_messages_up.sql
- [ ] T005 Create migration (down): messages table at migrations_sqlx/20251106_02_wire_messages_down.sql
- [ ] T006 Create migration: conversations table at migrations_sqlx/20251106_03_wire_conversations_up.sql
- [ ] T007 Create migration (down): conversations table at migrations_sqlx/20251106_03_wire_conversations_down.sql
- [ ] T008 Add config knobs in crates/server/src/config.rs (batch_size, claim_timeout_secs, max_retries, backoff_base_ms)
- [ ] T009 [P] Extend metrics in crates/server/src/metrics.rs for worker_queued, worker_claimed, worker_processed, worker_error, worker_dead_letter, and latency

## Phase 2 – Foundational

- [ ] T010 Create DB module root crates/server/src/store_db/mod.rs
- [ ] T011 Create inbound events store crates/server/src/store_db/inbound_events.rs (insert, claim_batch, mark_processed, mark_error, reap_stale)
- [ ] T012 Create messages store crates/server/src/store_db/messages.rs (insert_from_inbound, helpers)
- [ ] T013 Create conversations store crates/server/src/store_db/conversations.rs (upsert_on_message, list, list_messages)
- [ ] T014 [P] Add shared normalization helper crates/server/src/store_db/normalize.rs (channel-aware address normalization and conversation key)

## Phase 3 – [US1] Persist inbound events (P1)

- [ ] T015 [US1] Implement insert_inbound_event in crates/server/src/store_db/inbound_events.rs using SQLx and idempotent upsert on (channel, provider_message_id)
- [ ] T016 [US1] Wire provider webhooks to DB: update crates/server/src/api/provider_mock.rs to persist inbound_events instead of in-memory
- [ ] T017 [US1] Wire real webhooks to DB: update crates/server/src/api/messages.rs (inbound/webhooks handlers) to persist inbound_events
- [ ] T018 [P] [US1] Add tracing (mock=true, event ids) in provider inbound handlers crates/server/src/api/provider_mock.rs
- [ ] T019 [US1] Verify duplicate inbound (same provider id) returns 202 without duplicate rows (graceful unique violation handling)

## Phase 4 – [US2] Background worker processes events (P1)

- [ ] T020 [US2] Add worker module crates/server/src/worker/inbound.rs to claim with SELECT … FOR UPDATE SKIP LOCKED and set status=processing, processor_id
- [ ] T021 [US2] Implement processing path: create Message rows and mark inbound_events processed in crates/server/src/worker/inbound.rs
- [ ] T022 [P] [US2] Implement retry policy with exponential backoff (attempt_count, next_attempt_at) and mark dead_letter after max_retries in crates/server/src/worker/inbound.rs
- [ ] T023 [US2] Add metrics increments and timings in worker using crates/server/src/metrics.rs
- [ ] T024 [US2] Wire worker startup in crates/server/src/lib.rs (spawn background task; read config knobs)

## Phase 5 – [US3] Conversations read from DB (P2)

- [ ] T025 [US3] Implement conversations list using DB: crates/server/src/store_db/conversations.rs (aggregate by key, order by last_activity_at desc)
- [ ] T026 [US3] Implement messages list using DB: crates/server/src/store_db/conversations.rs (order by timestamp asc, pagination)
- [ ] T027 [P] [US3] Update API handlers to use DB stores: crates/server/src/api/conversations.rs (list_conversations, list_messages)

## Phase 6 – Polish & Cross-cutting

- [ ] T028 Document DB quickstart and verify steps in specs/007-wire-postgresql-store/quickstart.md (psql queries, config)
- [ ] T029 Update README anchors to reference this feature’s quickstart (optional) at README.md
- [ ] T030 [P] Update tests to assert conversations non-empty after activity: tests/http/tests.json (add "assert": ".items | length > 0" to the List conversations case)
- [ ] T031 Ensure SQLX_OFFLINE compatibility for CI builds (sqlx-data.json refresh if needed)
- [ ] T032 Add basic health/metrics note for worker throughput in README.md

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
