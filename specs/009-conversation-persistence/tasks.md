---

description: "Tasks for Conversation Persistence & Unification"
---

# Tasks: Conversation Persistence & Unification

**Input**: Design documents from `/specs/009-conversation-persistence/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Only include tests explicitly requested in the specification. This feature requests deterministic fixtures and concurrency/load checks; include focused unit/integration tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish directories, configs, and scaffolding referenced by the plan

- [x] T001 Ensure migrations folder exists at crates/db-migrate/migrations_sqlx/
- [x] T002 Ensure server config directory present at crates/server/config/
- [x] T003 [P] Add config key default for snippet length in crates/server/config (e.g., conversations.snippet_length=64)
- [x] T004 [P] Create module folder crates/core/src/conversations/ for normalization and key derivation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Schema and core plumbing needed across all stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Create SQL migration: conversations table and unique index in crates/db-migrate/migrations_sqlx/0014_add_conversation_key_columns.up.sql
- [x] T006 Create SQL migration (down): drop conversations in crates/db-migrate/migrations_sqlx/0014_add_conversation_key_columns.down.sql
- [x] T007 Create SQL migration: add messages.conversation_id (FK NOT NULL) and supporting index in crates/db-migrate/migrations_sqlx/0015_add_messages_conversation_fk.up.sql
- [x] T008 Create SQL migration (down): drop messages.conversation_id FK in crates/db-migrate/migrations_sqlx/0015_add_messages_conversation_fk.down.sql
- [x] T009 [P] Add timestamp index on conversations.last_activity_at in crates/db-migrate/migrations_sqlx/0014_add_conversation_key_columns.up.sql
- [x] T010 [P] Add direction CHECK (Inbound/Outbound) if missing in crates/db-migrate/migrations_sqlx/0016_add_message_direction_check.up.sql
- [ ] T011 [P] Implement normalization: email lowercasing + plus-tag equivalence in crates/core/src/conversations/normalize_email.rs
- [ ] T012 [P] Implement normalization: phone digits + optional leading plus in crates/core/src/conversations/normalize_phone.rs
- [ ] T013 Implement canonical key + participant ordering in crates/core/src/conversations/key.rs
- [ ] T014 Unit tests for normalization and key derivation in crates/core/src/conversations/tests.rs
  
- [x] T011 [P] Implement normalization: email lowercasing + plus-tag equivalence in crates/core/src/conversations/normalize_email.rs
- [x] T012 [P] Implement normalization: phone digits + optional leading plus in crates/core/src/conversations/normalize_phone.rs
- [x] T013 Implement canonical key + participant ordering in crates/core/src/conversations/key.rs
- [x] T014 Unit tests for normalization and key derivation in crates/core/src/conversations/*

**Checkpoint**: Foundation ready — migrations authored; normalization and key derivation implemented with tests

---

## Phase 3: User Story 1 - Durable, unified conversations (Priority: P1) 🎯 MVP

**Goal**: Every message (inbound/outbound) is linked to a single durable conversation with atomic counters and last activity updates

**Independent Test**: Send an outbound and inbound pair for same participants/channel; verify a single conversation with message_count=2 and correct last_activity

### Tests for User Story 1

- [X] T015 [P] [US1] Deterministic fixture: outbound + inbound pair results in one conversation at tests/integration/conversation_flow.rs
- [X] T016 [P] [US1] Concurrency test: 100 inserts same key → 1 conversation, count=100 at tests/integration/concurrency_upsert.rs

### Implementation for User Story 1

- [X] T017 [P] [US1] Conversation upsert function with INSERT .. ON CONFLICT in crates/core/src/conversations/upsert.rs
- [X] T018 [US1] Integrate upsert into message persistence path (inbound/outbound) in crates/core/src/messaging/mod.rs
- [X] T019 [US1] Ensure atomic update of message_count and last_activity with message insert in crates/core/src/conversations/upsert.rs
- [X] T020 [US1] Idempotency handling: do not inflate count on duplicate message in crates/core/src/messaging/mod.rs
- [X] T021 [US1] Metrics counters: created/reused/failures in crates/core/src/conversations/metrics.rs
- [X] T022 [US1] Structured logging with message_id and conversation_key in crates/core/src/conversations/logging.rs
- [X] T023 [US1] Backfill utility skeleton in crates/db-migrate/src/backfill_conversations.rs
  
- [x] T017 [P] [US1] Conversation upsert function with INSERT .. ON CONFLICT in crates/core/src/conversations/upsert.rs
- [x] T021 [US1] Metrics counters: created/reused/failures in crates/core/src/conversations/metrics.rs

**Checkpoint**: US1 functional — one conversation per participant pair/channel, atomic counters, tests passing

---

## Phase 4: User Story 2 - Deterministic conversation listing (Priority: P2)

**Goal**: List conversations from DB with deterministic ordering and pagination; remove in-memory fallback when DB is available

**Independent Test**: Repeated list calls return identical order; pagination stable; includes participants and key

### Tests for User Story 2

- [X] T024 [P] [US2] Contract test for GET /api/conversations in tests/contract/conversations_list.rs

### Implementation for User Story 2

- [X] T025 [P] [US2] Extend Conversation DTO to include participant_a, participant_b, key in crates/server/src/api/conversations.rs
- [X] T026 [US2] Implement DB-sourced listing ordered by last_activity_at DESC, id DESC in crates/server/src/api/conversations.rs
- [ ] T027 [US2] Remove in-memory fallback when DB is configured in crates/server/src/api/conversations.rs
- [ ] T028 [US2] Add paging parameters validation and next_page computation in crates/server/src/api/conversations.rs

**Checkpoint**: US2 functional — deterministic listing from persistence with proper DTOs

---

## Phase 5: User Story 3 - Conversation messages with safe snippets (Priority: P3)

**Goal**: Return messages within a conversation with accurate from/to and UTF-8 safe snippets respecting configured length

**Independent Test**: Messages containing multi-byte characters have correctly truncated snippets without broken code points

### Tests for User Story 3

- [X] T029 [P] [US3] Contract test for GET /api/conversations/{id}/messages in tests/contract/conversation_messages.rs
- [X] T030 [P] [US3] UTF-8 snippet boundary test in tests/unit/snippet_unicode.rs

### Implementation for User Story 3

- [X] T031 [P] [US3] Implement snippet utility with Unicode-safe truncation in crates/core/src/conversations/snippet.rs
- [X] T032 [US3] Expose snippet length config and wire into handler in crates/server/src/config/mod.rs
- [X] T033 [US3] Update conversation messages handler to include from/to and snippet in crates/server/src/api/conversations.rs
- [X] T034 [US3] Ensure timestamp ordering uses received_at for inbound, sent_at for outbound in crates/server/src/api/conversations.rs

**Checkpoint**: US3 functional — message listing with correct addresses and safe snippets

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalization tasks spanning multiple user stories

- [ ] T035 [P] Backfill implementation: batch NULL conversation_id → upsert + set FK; aggregate recompute in crates/db-migrate/src/backfill_conversations.rs
- [ ] T036 [P] Add verification query/report after backfill in crates/db-migrate/src/backfill_conversations.rs
- [X] T037 Documentation updates per quickstart and normalization rules in README.md and specs/009-conversation-persistence/quickstart.md
- [ ] T038 Observability: ensure metrics exported and dashboards/alerts updated in docs/observability.md
- [ ] T039 Feature toggle or gating for legacy in-memory store in crates/core/src/conversations/mod.rs
- [ ] T040 Final load test sweep and latency report script in tests/integration/load_report.rs

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1): No dependencies — can start immediately
- Foundational (Phase 2): Depends on Setup — BLOCKS all user stories
- User Stories (Phases 3–5): All depend on Foundational; then can proceed in parallel or P1→P2→P3
- Polish (Phase 6): Depends on completion of targeted user stories (at minimum US1) and migrations

### User Story Dependencies

- US1 (P1): After Phase 2; no dependency on other stories
- US2 (P2): After Phase 2; independent of US1, but uses same entities
- US3 (P3): After Phase 2; independent of US1/US2, uses shared utilities

### Parallel Opportunities

- Setup: T003, T004 in parallel
- Foundational: T009–T012 in parallel; T011 and T012 parallel to each other; T013 after normalizers
- US1: T015 and T016 in parallel; T017 parallel to T021/T022; T018 after T017
- US2: T024 contract test in parallel with DTO extension (T025)
- US3: T029 and T030 in parallel; T031 parallel to T032

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Setup → Foundational
2. Implement US1 upsert + integration + metrics + logging
3. Validate with integration tests (T015, T016)
4. Deliver MVP

### Incremental Delivery

- Add US2 deterministic listing → validate contract → deliver
- Add US3 snippets + messages listing → validate contract → deliver

---

## Validation: Checklist Format

All tasks in this document follow the strict format:

- Checkbox `- [ ]`
- Task ID `T###`
- Optional `[P]` marker for parallelizable tasks
- `[US#]` label present ONLY for user-story phases
- Each task includes a precise file path
