# Tasks: 003-poc-sqlx-migration

This checklist is generated from plan/spec/research/data-model. Tasks are organized by phases and user stories. Tests are optional for this PoC; focus is on runnable migrations and utility.

## Phase 1 — Setup

- [X] T001 Update workspace members to include db-migrate in `Cargo.toml`
- [X] T002 Move crate folder `crates/admin` → `crates/db-migrate` and update package name in `crates/db-migrate/Cargo.toml`
- [X] T003 Add SQLx and migrations deps in `crates/db-migrate/Cargo.toml` (features: postgres, runtime-tokio, macros)
- [X] T004 Create migrations directory `crates/db-migrate/migrations/` (empty placeholder committed)
- [X] T005 Implement migration runner entrypoint in `crates/db-migrate/src/main.rs` (apply/create subcommands)
- [X] T006 Add Makefile targets in `Makefile` for `migrate-apply` and `migrate-new`
- [X] T007 Update quickstart with exact commands in `specs/003-poc-sqlx-migration/quickstart.md`

## Phase 2 — Foundational (shared prerequisites)

- [X] T008 Create core tables migration in `crates/db-migrate/migrations/0001_create_core_tables.sql`
- [X] T009 Create dedup tables migration in `crates/db-migrate/migrations/0002_create_dedup_tables.sql`
- [X] T010 Add baseline indexes in `crates/db-migrate/migrations/0003_add_indexes.sql`
- [X] T011 Add audit/UTC triggers in `crates/db-migrate/migrations/0004_triggers_audit_utc.sql`

## Phase 3 — [US1] Operator can run migrations (P1)

Story Goal: Provide a reliable CLI to create and apply SQLx migrations against DATABASE_URL.

Independent Test Criteria:
- Running `apply` connects and applies all pending migrations; prints applied list; exits 0.
- Invalid DATABASE_URL or SQL error exits non-zero with clear message.
- `new <name>` creates timestamped up/down files in migrations directory.

- [X] T012 [US1] Wire `sqlx::migrate!` to load `crates/db-migrate/migrations` in `crates/db-migrate/src/main.rs`
- [X] T013 [US1] Implement `apply` subcommand with DATABASE_URL in `crates/db-migrate/src/main.rs`
- [X] T014 [US1] Implement `new <name>` subcommand to scaffold migration files in `crates/db-migrate/src/main.rs`
- [X] T015 [US1] Add friendly error messages and non-zero exit on failure in `crates/db-migrate/src/main.rs`
- [X] T016 [US1] Add Make targets `migrate-apply` and `migrate-new` in `Makefile`
- [X] T017 [US1] Document migration usage flow in `specs/003-poc-sqlx-migration/quickstart.md`

## Phase 4 — [US2] Developer can insert messages grouped into conversations (P1)

Story Goal: Enable inserting participants and messages linked to conversations with proper constraints.

Independent Test Criteria:
- Inserts of customers, contacts, conversations, participants, and messages succeed with FKs enforced.
- Conversation participants primary key prevents duplicate roles for same contact/conversation.
- Query by conversation_id returns messages ordered by sent_at desc via index.

- [X] T018 [US2] Ensure FK constraints and NOT NULLs in core schema in `crates/db-migrate/migrations/0001_create_core_tables.sql`
- [X] T019 [US2] Add participant linking via `conversation_participants` with PK in `crates/db-migrate/migrations/0001_create_core_tables.sql`
- [X] T020 [US2] Add `messages(conversation_id, sent_at DESC)` index in `crates/db-migrate/migrations/0003_add_indexes.sql`
- [X] T021 [US2] Add sample insert SQL snippet to docs in `specs/003-poc-sqlx-migration/data-model.md`

## Phase 5 — [US3] Analyst can query conversation overview view (P2)

Story Goal: Provide views to summarize conversations and list messages with body and attachment counts.

Independent Test Criteria:
- `conversation_overview` returns conversation_id, customer_id, last_message_at, message_count, participant_count.
- `conversation_messages` returns expected columns and joins body text via application query pattern.

- [X] T022 [US3] Create `conversation_overview` view in `crates/db-migrate/migrations/0005_views.sql`
- [X] T023 [US3] Create `conversation_messages` view in `crates/db-migrate/migrations/0005_views.sql`
- [X] T024 [US3] Describe expected columns and filters in `specs/003-poc-sqlx-migration/data-model.md`

## Phase 6 — [US4] Integrator relies on dedup tables (P1)

Story Goal: Ensure dedup tables uniquely store normalized items keyed by 64-bit hash.

Independent Test Criteria:
- Inserting same normalized item twice violates UNIQUE(hash) or gracefully upserts via application.
- Normalization triggers (where present) produce stable normalized forms (emails lowercased; phones digit/E.164 best-effort).

- [X] T025 [US4] Define UNIQUE(hash) and required columns for dedup tables in `crates/db-migrate/migrations/0002_create_dedup_tables.sql`
- [X] T026 [US4] Add normalization triggers stubs (email/phone/body) in `crates/db-migrate/migrations/0004_triggers_audit_utc.sql`
- [X] T027 [US4] Document normalization contract (app computes hash; triggers may normalize) in `specs/003-poc-sqlx-migration/research.md`

## Phase 7 — [US5] Backend service queue PoC (P2)

Story Goal: Provide a simple queue table for webhook ingestion with a safe dequeue pattern.

Independent Test Criteria:
- Insert into `inbound_events` succeeds with defaults for received_at/attempts/status.
- Dequeue using `FOR UPDATE SKIP LOCKED` returns rows and prevents double-processing across sessions.

- [X] T028 [US5] Create `inbound_events` table with indexes in `crates/db-migrate/migrations/0006_queue_inbound_events.sql`
- [X] T029 [US5] Document dequeue pattern (FOR UPDATE SKIP LOCKED) in `specs/003-poc-sqlx-migration/data-model.md`

## Final Phase — Polish & Cross-Cutting

- [X] T030 Add SQLX_OFFLINE CI hint and env docs in `README.md`
- [ ] T031 Add `.jjignore`/`.gitignore` updates for migration artifacts if needed in `/.jjignore`
- [X] T032 Add `DATABASE_URL` example to `.env.example` in `/.env.example`
- [X] T033 Add CHANGELOG entry for schema init in `CHANGELOG.md`

---

## Dependencies (User Stories)

- Order: US1 → (US2 and US4 in parallel) → US3 → US5
  - US1 (migration utility) unblocks all other stories
  - US2 (core schema) and US4 (dedup tables) can proceed in parallel after US1
  - US3 (views) depends on US2
  - US5 (queue) independent once US1 completes

## Parallel Execution Examples

- During US1: T012–T017 can be split — CLI wiring (T012–T015) in parallel with Make/doc updates (T016–T017)
- After US1: US2 (T018–T021) and US4 (T025–T027) can be done concurrently (distinct files)
- Views (T022–T024) can run alongside queue (T028–T029)

## Implementation Strategy

- MVP: Deliver US1 (db-migrate apply/new) with Foundational migrations creating core + dedup tables (T008–T011). Defer views and queue.
- Incremental: Add views (US3) and queue (US5) next; refine normalization triggers and indexes as data arrives.
