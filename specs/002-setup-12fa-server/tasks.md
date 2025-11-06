# Tasks — Setup 12‑Factor Server Bootstrap (002-setup-12fa-server)

This checklist is generated from the spec and plan. Complete tasks top-to-bottom within each phase. [P] marks tasks that can run in parallel (different files, no unmet dependencies).

## Phase 1 — Setup (project initialization)

- [ ] T001 Create Cargo workspace root Cargo.toml with [workspace] members at repository root
- [ ] T002 Create crates/core/Cargo.toml and crates/core/src/lib.rs (library crate scaffold)
- [ ] T003 Create crates/server/Cargo.toml and crates/server/src/main.rs (binary crate scaffold)
- [ ] T004 Create crates/admin/Cargo.toml and crates/admin/src/main.rs (binary crate scaffold)
- [ ] T005 [P] Add .env.example at repository root documenting PORT, HEALTH_PATH, LOG_LEVEL
- [ ] T006 [P] Update Makefile with run targets (server) and lint-shell consolidation
- [ ] T007 [P] Add contracts reference in README.md and link to specs/002-setup-12fa-server/contracts/openapi.yaml

## Phase 2 — Foundational (blocking prerequisites)

- [ ] T008 Implement configuration module in crates/core/src/config.rs (env + dotenvy precedence, defaults)
- [ ] T009 Implement logging init in crates/core/src/logging.rs using tracing + tracing-subscriber
- [ ] T010 [P] Expose core prelude in crates/core/src/lib.rs (pub use config, logging)
- [ ] T011 [P] Add unit tests for config defaults/validation in crates/core/src/config.rs (rstest)
- [ ] T012 [P] Add unit tests for logging init level handling in crates/core/src/logging.rs (rstest)

## Phase 3 — User Story 1 (P1): Start service and verify health

- [ ] T013 [US1] Wire server main to load config and init logging in crates/server/src/main.rs
- [ ] T014 [US1] Add Axum/Tokio server binding on configured PORT in crates/server/src/main.rs
- [ ] T015 [US1] Implement GET health route at HEALTH_PATH returning {"status":"ok"} in crates/server/src/main.rs
- [ ] T016 [P] [US1] Add integration test that starts server on ephemeral port and asserts 200/JSON payload in crates/server/tests/health.rs
- [ ] T017 [P] [US1] Emit startup and request logs; assert presence in test or via capturing logs in crates/server/tests/health.rs
- [ ] T018 [US1] Update specs/002-setup-12fa-server/quickstart.md with run instructions and curl example (actual ports/paths)

## Phase 4 — User Story 2 (P2): Configuration precedence and defaults

- [ ] T019 [US2] Implement precedence (env > .env > defaults) in crates/core/src/config.rs with explicit ordering
- [ ] T020 [P] [US2] Add tests for precedence conflicts (LOG_LEVEL, PORT) in crates/core/src/config.rs
- [ ] T021 [P] [US2] Add tests for defaults when unset in crates/core/src/config.rs
- [ ] T022 [US2] Log resolved config and source precedence at startup in crates/server/src/main.rs

## Phase 5 — User Story 3 (P3): Graceful shutdown and error handling

- [ ] T023 [US3] Implement graceful shutdown on SIGINT/SIGTERM with bounded timeout in crates/server/src/main.rs
- [ ] T024 [P] [US3] Add test to simulate signal and verify shutdown logs/exit in crates/server/tests/shutdown.rs
- [ ] T025 [US3] Detect port-in-use error path; exit non-zero with clear message in crates/server/src/main.rs
- [ ] T026 [P] [US3] Add test: occupy port then assert startup fails non-zero with clear error in crates/server/tests/port_conflict.rs

## Final Phase — Polish & Cross-Cutting

- [ ] T027 Sync health path in contracts/openapi.yaml to match HEALTH_PATH default in specs/002-setup-12fa-server/contracts/openapi.yaml
- [ ] T028 Add sample .env with commented defaults at repository root
- [ ] T029 Ensure sensitive values (future) are redacted in logs; add note in crates/core/src/logging.rs
- [ ] T030 Update README.md Quickstart and Troubleshooting for JJ bookmarks and cargo workspace layout

## Dependencies and Story Order

- Phase 1 → Phase 2 → US1 → US2 → US3 → Polish
- US1 is the MVP slice; US2 and US3 can proceed after foundational pieces, but US2 depends on config module; US3 depends on server main from US1.

## Parallelization Examples

- T005/T006/T007 can run in parallel during setup.
- In Foundational, T010/T011/T012 can run in parallel after T008/T009 are drafted.
- In US1, testing tasks T016/T017 can run in parallel after T014/T015 stubs exist.
- In US2 and US3, tests (T020/T021/T024/T026) can run in parallel once the main code paths are stubbed.

## Implementation Strategy (MVP first)

- Implement Phase 1 and Phase 2, then deliver US1 end-to-end (boot + health + logs). This is the MVP and unblocks CI, monitoring hooks, and containerization. Follow with US2 (config precedence) and US3 (graceful shutdown, error paths), then polish.