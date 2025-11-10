---

description: "Task list for Feature 011: Go System Conversion"
---

# Tasks: Go System Conversion (Feature 011)

**Input**: Design documents from `/specs/011-go-system-conversion/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md (decisions), data-model.md (TBD), contracts/ (TBD)

**Tests**: Tests are optional per process. This feature’s spec defines independent test criteria; we include them as prose under each story. Add explicit test tasks if you choose a TDD approach later.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Project shape per plan:

```text
go/
  cmd/server/
  internal/ (server, middleware, db, providers, metrics, logging, security)
  api/ (handlers, models)
```

---

## Phase 1: Setup (Shared Infrastructure)

Purpose: Project initialization and basic structure.

 - [X] T001 Create Go module tree at go/ (cmd/server, internal, api)
 - [X] T002 Initialize Go module in go/go.mod
 - [X] T003 [P] Create entrypoint in go/cmd/server/main.go (chi router stub + /healthz)
 - [X] T004 [P] Add container build file go/Dockerfile (multi-stage build)
 - [X] T005 [P] Add Go targets to Makefile (build/test/lint/package)
 - [X] T006 [P] Add CI workflow for Go in .github/workflows/go.yml (build/test)
 - [X] T007 [P] Add migration helper script bin/migrate.sh (wrap golang-migrate CLI)
 - [X] T008 [P] Add lint configuration .golangci.yml (go vet, staticcheck, fmt)

---

## Phase 2: Foundational (Blocking Prerequisites)

Purpose: Core infrastructure that MUST be complete before ANY user story can be implemented.

- [ ] T009 Setup migrations integration in go/internal/db/migrate/migrate.go (invoke CLI on startup/CI)
- [ ] T010 [P] Implement configuration loader in go/internal/config/config.go (env→struct; rate limits, headers, SSRF allowlist)
- [ ] T011 [P] Setup router structure in go/internal/server/router.go (chi + middleware pipeline)
- [ ] T012 [P] Implement logging init in go/internal/logging/logging.go (zap + redaction)
- [ ] T013 [P] Implement metrics init in go/internal/metrics/metrics.go (Prometheus registry + /metrics)
- [ ] T014 [P] Implement security headers middleware in go/internal/middleware/security_headers.go
- [ ] T015 [P] Implement rate limiter middleware in go/internal/middleware/rate_limit.go (x/time/rate)
- [ ] T016 [P] Implement SSRF allowlist validator in go/internal/security/egress_validator.go
- [ ] T017 [P] Define secrets abstraction in go/internal/secrets/secrets.go (interface)
- [ ] T018 [P] Implement Vault client in go/internal/secrets/vault.go
- [ ] T019 [P] Implement dev secrets stub in go/internal/secrets/dev.go (explicitly flagged)
- [ ] T020 [P] Add circuit breaker wrapper in go/internal/resilience/breaker.go (sony/gobreaker)
- [ ] T021 Add health and metrics endpoints in go/api/health.go and go/api/metrics.go
- [ ] T022 Wire startup in go/cmd/server/main.go (config/logging/metrics/router/migrations)

Checkpoint: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - API Parity (Priority: P1) 🎯 MVP

Goal: Consumers can use the new service without client changes; endpoints, shapes, and status codes remain compatible.

Independent Test: Contract tests pass using existing client payloads and expectations; health and metrics endpoints respond.

### Implementation for User Story 1

- [ ] T023 [P] [US1] Define request/response models in go/api/models/messages.go (match existing JSON schemas)
- [ ] T024 [P] [US1] Define conversation models in go/api/models/conversations.go
- [ ] T025 [US1] Implement public message endpoints in go/api/messages.go (route handlers)
- [ ] T026 [US1] Implement conversation endpoints in go/api/conversations.go (route handlers)
- [ ] T027 [US1] Wire public routes in go/internal/server/router.go (depends on T025, T026)
- [ ] T028 [US1] Implement error mapping helpers in go/internal/server/errors.go (status codes/body shape)
- [ ] T029 [US1] Ensure status code parity in go/api/messages.go and go/api/conversations.go

Checkpoint: User Story 1 independently functional and demonstrable

---

## Phase 4: User Story 2 - Security Parity (Priority: P1)

Goal: Enforce equivalent security controls (auth/authz, headers, rate limits, SSRF) so risk posture does not regress.

Independent Test: Security suite verifies headers, 401/403, SSRF allowlist, and rate limiting.

### Implementation for User Story 2

- [ ] T030 [P] [US2] Implement authentication middleware in go/internal/middleware/auth.go (policy-driven)
- [ ] T031 [US2] Implement session expiry and failure backoff in go/internal/middleware/auth.go
- [ ] T032 [US2] Apply security headers middleware in go/internal/server/router.go for protected routes
- [ ] T033 [US2] Enforce SSRF validation in go/internal/http/client.go (egress wrapper)
- [ ] T034 [US2] Apply rate limits to protected endpoints in go/internal/server/router.go
- [ ] T035 [US2] Add security config to go/internal/config/config.go (auth, headers, allowlists, rate limits)

Checkpoint: User Story 1 and 2 independently functional and demonstrable

---

## Phase 5: User Story 3 - Operability Parity (Priority: P1)

Goal: Build, deploy, and monitor with existing workflows (container images, compose, health, metrics, structured logs).

Independent Test: Service runs under Compose; health/metrics/logs verified.

### Implementation for User Story 3

- [X] T036 [P] [US3] Add Go service stanza to docker-compose.yml (side-by-side with Rust service)
- [ ] T037 [P] [US3] Expose /healthz and /metrics in go/internal/server/router.go
- [ ] T038 [US3] Ensure structured logs + redaction in go/internal/logging/logging.go
- [ ] T039 [US3] Add optional pprof in go/internal/server/pprof.go (guarded by env)
- [ ] T040 [US3] Document operations in specs/011-go-system-conversion/quickstart.md (build/run/verify)

Checkpoint: User Story 1–3 independently functional and demonstrable

---

## Phase 6: User Story 4 - Data Access Parity (Priority: P2)

Goal: Equivalent validated and parameterized data operations using same schema and transactional semantics.

Independent Test: Read/write flows produce identical results in controlled scenarios.

### Implementation for User Story 4

- [ ] T041 [P] [US4] Add SQLC config in go/sqlc.yaml (pgx driver)
- [ ] T042 [P] [US4] Place SQL queries in go/queries/*.sql (mirror existing SQL where applicable)
- [ ] T043 [US4] Generate SQLC code into go/internal/db/generated/
- [ ] T044 [US4] Implement messages repository in go/internal/db/repository/messages_repository.go
- [ ] T045 [US4] Implement conversations repository in go/internal/db/repository/conversations_repository.go
- [ ] T046 [US4] Implement transaction helpers in go/internal/db/tx.go (semantic parity)
- [ ] T047 [US4] Integrate repositories with handlers in go/api/messages.go and go/api/conversations.go

Checkpoint: All user stories now independently functional

---

## Phase N: Polish & Cross-Cutting Concerns

Purpose: Improvements that affect multiple user stories

- [ ] T048 [P] Add SBOM generation (cyclonedx-gomod) in .github/workflows/go.yml
- [ ] T049 [P] Add vulnerability scanning (govulncheck) in .github/workflows/go.yml
- [ ] T050 [P] Add license compliance (go-licenses) in .github/workflows/go.yml
- [ ] T051 [P] Add metrics mapping doc in specs/011-go-system-conversion/contracts/metrics-mapping.md
- [ ] T052 Code cleanup and refactoring across go/
- [ ] T053 [P] Validate quickstart.md end-to-end in specs/011-go-system-conversion/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1): No dependencies – start immediately
- Foundational (Phase 2): Depends on Setup – BLOCKS all user stories
- User Stories (Phase 3+): Depend on Foundational; can proceed in parallel after Phase 2
- Polish: Depends on all desired user stories being complete

### User Story Dependencies

- User Story 1 (P1): After Foundational – no other story dependencies
- User Story 2 (P1): After Foundational – independent but uses middleware/hooks
- User Story 3 (P1): After Foundational – independent; focuses on ops surfaces
- User Story 4 (P2): After Foundational – independent; focuses on DB parity

### Within Each User Story

- Models → Services/Repositories → Endpoints → Integration
- Error mapping and status codes finalized before demos

### Parallel Opportunities

- Setup: T003–T008 can run in parallel
- Foundational: T010–T020 can run in parallel; T021–T022 follow
- US1: T023–T026 can run in parallel; T027–T029 follow
- US2: T030 parallel with T032–T035; T031 follows T030
- US3: T036–T039 can run in parallel; T040 follows
- US4: T041–T045 can run in parallel; T046–T047 follow
- Polish: T048–T051 can run in parallel

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL – blocks all stories)
3. Complete Phase 3: User Story 1 (API Parity)
4. STOP and VALIDATE: Test User Story 1 independently
5. Demo/iterate

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Test → Demo
3. Add US2 → Test → Demo
4. Add US3 → Test → Demo
5. Add US4 → Test → Demo

### Parallel Team Strategy

- After Foundational: split US1, US2, US3 concurrently; bring in US4 once DB access layer is ready

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to user story
- Each user story should be independently demonstrable
- Avoid cross-story coupling; keep file boundaries clean
