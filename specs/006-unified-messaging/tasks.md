# Tasks: Unified Messaging (Feature 006)

This task plan is generated per the speckit.tasks workflow and organized by user story. Each task uses the strict checklist format.

## Phase 1 — Setup
- [x] T001 Create API module skeletons in crates/server/src/api/ (messages.rs, provider_mock.rs, conversations.rs)
- [x] T002 Create providers module for mocks in crates/server/src/providers/mock.rs
- [x] T003 Create queue modules for outbound worker in crates/server/src/queue/outbound.rs
- [x] T004 Create in-memory stores in crates/server/src/store/messages.rs and crates/server/src/store/conversations.rs
- [x] T005 Wire mod declarations in crates/server/src/api/mod.rs and crates/server/src/lib.rs

## Phase 2 — Foundational
- [x] T010 Add shared DTOs for messages in crates/server/src/types/message.rs
- [x] T011 Extend config for API_PROVIDER_* in crates/server/src/config.rs (timeout_pct, error_pct, ratelimit_pct, seed)
- [x] T012 Extend metrics counters in crates/server/src/metrics.rs (dispatch_attempts, dispatch_success, dispatch_rate_limited, dispatch_error, breaker_transitions)
- [x] T013 Update logging for dispatch/inbound paths in crates/server/src/middleware/logging.rs (reuse correlation IDs)

## Phase 3 — User Story 1: Send Outbound Messages via Unified API (P1)
Story goal: Accept SMS/MMS/Email requests, apply idempotency and rate limits, enqueue for dispatch.
Independent test criteria: 202 Accepted within 200ms; duplicate with same Idempotency-Key returns cached 202; 429 when rate limit exceeded.

- [x] T020 [US1] Define SmsRequest/EmailRequest validators in crates/server/src/types/message.rs
- [x] T021 [US1] Implement POST /api/messages/sms in crates/server/src/api/messages.rs
- [x] T022 [US1] Implement POST /api/messages/email in crates/server/src/api/messages.rs
- [x] T023 [P] [US1] Register routes in crates/server/src/lib.rs (Axum router wiring)
- [x] T024 [US1] Integrate idempotency cache in crates/server/src/api/messages.rs
- [x] T025 [US1] Enqueue outbound event to queue in crates/server/src/api/messages.rs

## Phase 4 — User Story 2: Process Inbound Provider-Originated Messages (P1)
Story goal: Accept inbound mock events, validate/normalize, and record in message store.
Independent test criteria: Valid inbound yields 202 and stored message; malformed inbound rejected once with error log.

- [x] T030 [US2] Implement POST /api/provider/mock/inbound in crates/server/src/api/provider_mock.rs
- [x] T031 [US2] Normalize inbound payloads in crates/server/src/types/message.rs
 - [x] T032 [P] [US2] Persist (in-memory) inbound message in crates/server/src/store/messages.rs

## Phase 5 — User Story 3: Provider Mock Fault & Rate Behavior (P2)
Story goal: Provide mock provider with configurable 200/429/5xx outcomes and deterministic seed; expose runtime config.
Independent test criteria: With configured failure mix, observe breaker open/half-open transitions and expected outcome ratios.

 - [x] T040 [US3] Implement mock provider with RNG + probabilities in crates/server/src/providers/mock.rs
- [x] T041 [US3] Add GET /api/provider/mock/config in crates/server/src/api/provider_mock.rs
- [x] T042 [US3] Add PUT /api/provider/mock/config in crates/server/src/api/provider_mock.rs
- [x] T043 [P] [US3] Read config defaults from env in crates/server/src/config.rs
 - [x] T044 [US3] Queue worker dispatch logic in crates/server/src/queue/outbound.rs (rate limiter + circuit breaker)
 - [x] T045 [US3] Increment metrics for outcomes and breaker transitions in crates/server/src/metrics.rs

## Phase 6 — User Story 4: Conversation Grouping (Stretch) (P3)
Story goal: Group messages by (channel, normalized_from, normalized_to); list conversations and messages.
Independent test criteria: Outbound + inbound pair appears as one conversation with correct order by timestamp.

- [ ] T050 [US4] Implement conversation key normalization in crates/server/src/store/conversations.rs
- [ ] T051 [US4] Upsert conversation on message ingest in crates/server/src/store/conversations.rs
- [ ] T052 [US4] Implement GET /api/conversations in crates/server/src/api/conversations.rs
- [ ] T053 [US4] Implement GET /api/conversations/{id}/messages in crates/server/src/api/conversations.rs
- [ ] T054 [P] [US4] Wire routes in crates/server/src/lib.rs

## Phase 7 — Polish & Cross-Cutting
- [ ] T060 Update specs/006-unified-messaging/quickstart.md with any new routes/flags
- [ ] T061 Link contracts and quickstart from README.md
- [ ] T062 Add Makefile target(s) for scenario runs (optional)

---

## Dependencies (Story Order)
1) US1 → 2) US2 → 3) US3 → 4) US4 (stretch)

Foundational (Phase 1–2) precede all user stories. US1 and US2 are both P1; US2 depends on foundational only and may proceed in parallel once Phase 2 is complete.

## Parallel Execution Examples
- T023 (router wiring) can run in parallel with T021/T022 once path names are agreed.
- T032 (persist inbound) can proceed in parallel with T030 (endpoint) with a mock store trait.
- T043 (env config) can run in parallel with T041/T042 (mock config endpoints).
- T054 (routes) can run in parallel with T052/T053 (conversations API) after path confirmation.

## Implementation Strategy
- MVP: Complete Phases 1–5 through US3 to validate send, receive, and resilience behavior without conversations.
- Incremental delivery: Ship US1 first (outbound send + queue), then US2 (inbound injection), then US3 (provider fault profiles). Add US4 (conversations) as a stretch.
