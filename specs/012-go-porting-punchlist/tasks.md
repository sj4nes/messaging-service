# Tasks: Go Porting Punchlist

**Input**: Design documents from `specs/012-go-porting-punchlist/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

## Phase 1: Setup (Shared Infrastructure)
**Purpose**: Ensure environment & baseline docs for audit exist.

- [X] T001 Create gap-inventory placeholder file specs/012-go-porting-punchlist/gap-inventory.md
- [X] T002 [P] Add README section referencing parity audit README.md
- [X] T003 [P] Verify seed initialization call exists in go/internal/db/seed/seed.go (no code change yet, inspection task)
- [X] T004 [P] Confirm list endpoints current serialization behavior in go/api/messages.go and go/internal/db/store/store.go
- [X] T005 Establish audit working notes file specs/012-go-porting-punchlist/notes.md

---

## Phase 2: Foundational (Blocking Prerequisites)
**Purpose**: Baseline normalization + metrics visibility required before gap enumeration.

- [ ] T006 Normalize list responses to empty arrays in go/internal/db/store/store.go
- [X] T007 [P] Add test assertion for empty array vs null in tests/http/tests.json
- [X] T008 [P] Document current metrics counters in specs/012-go-porting-punchlist/metrics-parity.md
- [X] T009 Add script snippet to quickstart for metrics curl specs/012-go-porting-punchlist/quickstart.md
- [X] T010 [P] Verify worker_processed handling and add comment clarifying simulation go/internal/metrics/metrics.go
- [X] T011 Confirm deterministic seed content (IDs/messages) and record in research.md (append update) specs/012-go-porting-punchlist/research.md

**Checkpoint**: Foundational parity prerequisites complete; gap audit can begin.

---

## Phase 3: User Story 1 - Identify Missing Parity Behaviors (Priority: P1) 🎯 MVP
**Goal**: Produce comprehensive gap inventory with classification.
**Independent Test**: gap-inventory.md lists categorized gaps with unique IDs and required fields; reviewable without implementing fixes.

### Implementation
- [ ] T012 [P] [US1] Enumerate functional gaps (API responses) in specs/012-go-porting-punchlist/gap-inventory.md
- [ ] T013 [P] [US1] Enumerate data gaps (seeding/persistence) in specs/012-go-porting-punchlist/gap-inventory.md
- [ ] T014 [P] [US1] Enumerate observability gaps (metrics/logs) in specs/012-go-porting-punchlist/gap-inventory.md
- [ ] T015 [P] [US1] Enumerate operability gaps (startup, fallback flags) in specs/012-go-porting-punchlist/gap-inventory.md
- [ ] T016 [US1] Assign priority & impact category to each gap specs/012-go-porting-punchlist/gap-inventory.md
- [ ] T017 [US1] Add reproduction steps & expected vs actual behavior specs/012-go-porting-punchlist/gap-inventory.md
- [ ] T018 [US1] Create JSON export matching gap-inventory.schema.json specs/012-go-porting-punchlist/gap-inventory.json
- [ ] T019 [US1] Validate JSON against schema using local validator script (add instructions) specs/012-go-porting-punchlist/quickstart.md
- [ ] T020 [US1] Add acceptance criteria per gap specs/012-go-porting-punchlist/gap-inventory.md

**Checkpoint**: gap-inventory artifacts ready (markdown + JSON).

---

## Phase 4: User Story 2 - Prioritize and Track Remediation (Priority: P2)
**Goal**: Convert gaps to remediation tasks with mapping & sequencing.
**Independent Test**: tasks mapping file exists with one TASK per gap and acceptance criteria; independent of fixes.

### Implementation
- [ ] T021 [P] [US2] Create remediation mapping file specs/012-go-porting-punchlist/remediation-tasks.md
- [ ] T022 [P] [US2] Generate TASK-### IDs aligned to GAP-### items specs/012-go-porting-punchlist/remediation-tasks.md
- [ ] T023 [US2] Add estimates and priority alignment specs/012-go-porting-punchlist/remediation-tasks.md
- [ ] T024 [US2] Define dependency graph (which tasks can run parallel) specs/012-go-porting-punchlist/remediation-tasks.md
- [ ] T025 [US2] Link acceptance criteria from gaps to tasks specs/012-go-porting-punchlist/remediation-tasks.md
- [ ] T026 [US2] Produce progress dashboard snippet (counts, percentages) specs/012-go-porting-punchlist/remediation-tasks.md
- [ ] T027 [P] [US2] Export tasks to JSON for tooling specs/012-go-porting-punchlist/remediation-tasks.json

**Checkpoint**: Remediation tasks fully defined.

---

## Phase 5: User Story 3 - Validate Closure & Parity Achievement (Priority: P3)
**Goal**: Verify gap closures & publish parity report.
**Independent Test**: parity-report JSON validates against schema with criticalRemaining=0 and closure markdown summary exists.

### Implementation
- [ ] T028 [P] [US3] Implement closure checklist specs/012-go-porting-punchlist/closure-checklist.md
- [ ] T029 [P] [US3] Execute contract tests post-remediation (record results) specs/012-go-porting-punchlist/closure-checklist.md
- [ ] T030 [US3] Compile parity-report JSON specs/012-go-porting-punchlist/parity-report.json
- [ ] T031 [US3] Validate parity-report against schema specs/012-go-porting-punchlist/parity-report.json
- [ ] T032 [US3] Generate markdown summary specs/012-go-porting-punchlist/parity-report.md
- [ ] T033 [US3] Confirm 95%+ closure & 0 critical remaining specs/012-go-porting-punchlist/parity-report.md
- [ ] T034 [US3] Document deferred low-impact gaps specs/012-go-porting-punchlist/parity-report.md

**Checkpoint**: Closure artifacts complete.

---

## Phase 6: Polish & Cross-Cutting Concerns
**Purpose**: Final refinements improving maintainability & transparency.

- [ ] T035 [P] Review documentation consistency (spec vs tasks vs quickstart) specs/012-go-porting-punchlist/quickstart.md
- [ ] T036 [P] Add ADR for worker deferral docs/adr/worker-deferral.md
- [ ] T037 Security scan confirmation entry in parity-report.md specs/012-go-porting-punchlist/parity-report.md
- [ ] T038 [P] Add script to generate parity report automatically bin/gen-parity-report.sh
- [ ] T039 Refactor any duplicated gap descriptions into shared snippets specs/012-go-porting-punchlist/gap-inventory.md
- [ ] T040 [P] Optional performance observation notes specs/012-go-porting-punchlist/perf-notes.md

---

## Dependencies & Execution Order
- Setup → Foundational → US1 (MVP) → US2 → US3 → Polish.
- Parallel: Enumeration subtasks (T012–T015) independent; remediation mapping creation tasks T021–T024 run after US1 checkpoint; closure tasks depend on remediation completion.

## Parallel Opportunities
- Enumeration tasks (gaps categories) parallelizable.
- Metrics/documentation tasks (T008, T010) parallel.
- Remediation mapping export (T027) parallel after mapping basics (T021–T024).
- Closure checklist & tests (T028, T029) parallel with report compilation (T030).

## MVP Scope
User Story 1 completion (Tasks T012–T020) plus Foundational (T006–T011) delivers auditable gap inventory.

## Format Validation
All tasks follow pattern: `- [ ] T### [P]? [US#]? Description with file path`.

## Counts
- Total tasks: 40
- US1 tasks: 9 (T012–T020)
- US2 tasks: 7 (T021–T027)
- US3 tasks: 7 (T028–T034)
- Setup + Foundational + Polish: 17

## Independent Test Criteria Summary
- US1: Presence & completeness of gap-inventory (unique IDs, categories, priorities, reproduction, acceptance).
- US2: Remediation mapping file with one task per gap, dependencies & estimates.
- US3: Parity report validates schema, no critical gaps remain.

## Implementation Strategy
Deliver MVP (US1) early; then map remediation (US2); finally closure (US3). Polish tasks can be deferred if closure timeline tight.
