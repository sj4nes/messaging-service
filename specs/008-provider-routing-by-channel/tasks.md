# Tasks: Provider Routing By Channel

**Input**: Design documents from `/specs/008-provider-routing-by-channel/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare folder/module structure for provider routing feature.

 - [X] T001 Create `crates/server/src/providers/` submodules `sms_mms.rs` and `email.rs`
 - [X] T002 Add provider registry module file `crates/server/src/providers/registry.rs`
 - [X] T003 [P] Add placeholder metrics label constants in `crates/server/src/metrics.rs`
 - [X] T004 [P] Ensure config parsing section ready for new env overrides in `crates/server/src/config.rs`
---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core abstractions and structures needed before user stories.

- [ ] T005 Define Provider trait in `crates/server/src/providers/registry.rs`
- [ ] T006 [P] Define OutboundMessage & DispatchResult structs in `crates/server/src/providers/registry.rs`
- [ ] T007 Extend `AppState` (file: `crates/server/src/lib.rs` or dedicated state module) with provider registry map
- [ ] T008 [P] Add per-provider config fields (sms/email) in `crates/server/src/config.rs` (env override keys)
- [ ] T009 Implement provider outcome probability helper reused by both providers in `crates/server/src/providers/common.rs`
- [ ] T010 Introduce per-provider circuit breaker storage (hash map) in `crates/server/src/state/idempotency.rs` (or new file `state/breakers.rs`)
- [ ] T011 Add invalid_routing counter in `crates/server/src/metrics.rs`
 - [X] T005 Define Provider trait in `crates/server/src/providers/registry.rs`
 - [X] T006 [P] Define OutboundMessage & DispatchResult structs in `crates/server/src/providers/registry.rs`
 - [X] T007 Extend `AppState` (file: `crates/server/src/lib.rs` or dedicated state module) with provider registry map
 - [X] T008 [P] Add per-provider config fields (sms/email) in `crates/server/src/config.rs` (env override keys)
 - [X] T009 Implement provider outcome probability helper reused by both providers in `crates/server/src/providers/common.rs`
 - [X] T010 Introduce per-provider circuit breaker storage (hash map) in `crates/server/src/state/idempotency.rs` (or new file `state/breakers.rs`)
 - [X] T011 Add invalid_routing counter in `crates/server/src/metrics.rs`

**Checkpoint**: Provider abstraction & registry ready; breakers isolated; proceed to story implementation.

---

## Phase 3: User Story 1 - Outbound Messages Route to Correct Provider (Priority: P1) 🎯 MVP

**Goal**: Route outbound SMS/MMS/Email to correct provider; record provider identity.
**Independent Test**: POST SMS/MMS/Email payloads; verify provider-specific log entries & metrics increments; 202 responses unchanged.

### Implementation

- [ ] T012 [P] [US1] Implement `SmsMmsMockProvider` in `crates/server/src/providers/sms_mms.rs`
- [ ] T013 [P] [US1] Implement `EmailMockProvider` in `crates/server/src/providers/email.rs`
- [ ] T014 [US1] Wire registry initialization in `crates/server/src/lib.rs` (AppState creation) mapping channels → providers
- [ ] T015 [US1] Refactor `queue/outbound.rs` to parse channel from event payload and select provider
- [ ] T016 [US1] Tag outbound message insert with `provider_name` in `crates/server/src/store/messages.rs`
- [ ] T017 [US1] Add structured log fields (provider_name, outcome) in `queue/outbound.rs`
- [ ] T018 [P] [US1] Add per-provider metrics counters (attempts, success, rate_limited, error) in `crates/server/src/metrics.rs`
- [ ] T019 [US1] Implement missing-provider 500 error path (log once) in `queue/outbound.rs`
- [ ] T020 [P] [US1] Unit test: registry mapping & routing in `crates/server/tests/provider_registry.rs`
- [ ] T021 [P] [US1] Integration test: SMS vs Email dispatch logs/metrics in `crates/server/tests/dispatch_routing.rs`
- [ ] T022 [US1] Update quickstart with provider routing verification steps in `specs/008-provider-routing-by-channel/quickstart.md`
 - [X] T012 [P] [US1] Implement `SmsMmsMockProvider` in `crates/server/src/providers/sms_mms.rs`
 - [X] T013 [P] [US1] Implement `EmailMockProvider` in `crates/server/src/providers/email.rs`
 - [X] T014 [US1] Wire registry initialization in `crates/server/src/lib.rs` (AppState creation) mapping channels → providers
 - [X] T015 [US1] Refactor `queue/outbound.rs` to parse channel from event payload and select provider
 - [X] T016 [US1] Tag outbound message insert with `provider_name` in `crates/server/src/store/messages.rs`
 - [X] T017 [US1] Add structured log fields (provider_name, outcome) in `queue/outbound.rs`
 - [X] T018 [P] [US1] Add per-provider metrics counters (attempts, success, rate_limited, error) in `crates/server/src/metrics.rs`
 - [X] T019 [US1] Implement missing-provider 500 error path (log once) in `queue/outbound.rs`
 - [X] T020 [P] [US1] Unit test: registry mapping & routing in `crates/server/tests/provider_registry.rs`
 - [X] T021 [P] [US1] Integration test: SMS vs Email dispatch logs/metrics in `crates/server/tests/dispatch_routing.rs`
 - [X] T022 [US1] Update quickstart with provider routing verification steps in `specs/008-provider-routing-by-channel/quickstart.md`

**Checkpoint**: MVP routing delivered and independently testable.

---

## Phase 4: User Story 2 - Provider Health Isolation & Circuit Breaking (Priority: P2)

**Goal**: Breaker isolation; failures in one provider do not affect others.
**Independent Test**: Force errors for sms-mms; confirm email unaffected and breaker transitions only for sms-mms.

### Implementation

- [ ] T023 [P] [US2] Add per-provider breaker lookup helper in `crates/server/src/providers/registry.rs`
- [ ] T024 [US2] Modify outbound worker to use provider-specific breaker (remove global assumption) in `crates/server/src/queue/outbound.rs`
- [ ] T025 [US2] Add breaker transition metrics (per provider) in `crates/server/src/metrics.rs`
- [ ] T026 [P] [US2] Unit test: breaker failure isolation in `crates/server/tests/breaker_isolation.rs`
- [ ] T027 [US2] Integration test: induced failures only open sms-mms breaker in `crates/server/tests/breaker_integration.rs`
- [ ] T028 [US2] Document breaker isolation test procedure in quickstart

**Checkpoint**: Isolation confirmed; resilience validated.

---

## Phase 5: User Story 3 - Deterministic Testing & Observability (Priority: P3)

**Goal**: Deterministic seeds produce reproducible outcome sequences; enhanced observability.
**Independent Test**: Fixed seeds yield identical ordered outcomes across runs.

### Implementation

- [ ] T029 [P] [US3] Add per-provider seed handling logic in `crates/server/src/providers/common.rs`
- [ ] T030 [US3] Extend provider initialization to apply seeds (one-time) in `crates/server/src/lib.rs`
- [ ] T031 [P] [US3] Unit test: deterministic sequence reproducibility in `crates/server/tests/deterministic.rs`
- [ ] T032 [US3] Integration test: mixed traffic reproducibility (seeded) in `crates/server/tests/deterministic_integration.rs`
- [ ] T033 [US3] Log seed presence at startup for audit in `crates/server/src/lib.rs`
- [ ] T034 [US3] Document deterministic testing steps in quickstart

**Checkpoint**: Deterministic behavior validated; observability enhancements complete.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements and validation.

- [ ] T035 [P] Add invalid_routing metric test in `crates/server/tests/invalid_routing.rs`
- [ ] T036 Refactor shared probability logic for clarity in `crates/server/src/providers/common.rs`
- [ ] T037 [P] Add doc comments for Provider trait & structs in `crates/server/src/providers/registry.rs`
- [ ] T038 Security/log review: verify no sensitive data in provider logs in `crates/server/src/queue/outbound.rs`
- [ ] T039 [P] Performance sanity test (benchmark dispatch loop) in `crates/server/tests/perf_dispatch.rs`
- [ ] T040 Update `CHANGELOG.md` entry for feature 008
- [ ] T041 Run quickstart validation steps end-to-end

---

## Dependencies & Execution Order

Phase dependencies:
- Phase 1 → Phase 2 → (Phases 3,4,5 can proceed sequentially or parallel after Phase 2)
- Phase 6 after all story phases targeted for delivery

User stories are independent after foundational; US1 delivers MVP; US2 and US3 build resilience & determinism.

Within each story: tests (parallel), then implementation tasks ordering where dependencies exist (e.g., registry before worker refactor).

## Parallel Opportunities

- Setup: T003, T004 can run parallel with T001/T002.
- Foundational: T006, T008, T009, T011 parallel; T005 precedes tasks using trait.
- US1: Provider impls (T012, T013) parallel; metrics (T018) parallel; tests (T020, T021) parallel before worker refactor finalize.
- US2: T023 & T025 parallel; tests T026 & T027 parallel after isolation code.
- US3: T029 & T031 parallel; integration test after seed wiring.
- Polish: Most tasks parallel except CHANGELOG (T040) and final validation (T041).

## Independent Test Criteria Summary

- US1: Distinct provider routing/logging/metrics for SMS vs Email.
- US2: Breaker isolation under induced sms-mms failures.
- US3: Identical seeded outcome sequences across runs.

## MVP Scope Recommendation

Deliver Phase 1 + Phase 2 + US1. Provides functional provider routing with identity recording; leaves resilience and determinism for later increments.

## Task Counts

- Total tasks: 41
- US1 tasks: 11 (T012–T022)
- US2 tasks: 6 (T023–T028)
- US3 tasks: 6 (T029–T034)
- Setup + Foundational: 11 (T001–T011)
- Polish: 7 (T035–T041)

## Format Validation

All tasks follow required format: `- [ ] TXXX [P?] [USn?] Description with file path`.
