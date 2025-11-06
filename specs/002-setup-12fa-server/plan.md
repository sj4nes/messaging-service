# Implementation Plan: Setup 12‑Factor Server Bootstrap

**Branch**: `002-setup-12fa-server` | **Date**: 2025-11-05 | **Spec**: specs/002-setup-12fa-server/spec.md  
**Input**: Feature specification from `/specs/002-setup-12fa-server/spec.md`

**Note**: This plan is generated per the project constitution and planning prompt and will drive Phase 0/1 artifacts.

## Summary

Bootstrap a minimal service aligned to 12‑Factor principles: load configuration from environment with optional .env, start an HTTP health endpoint on a configurable port/path, and emit structured logging for startup, requests, and shutdown. The codebase will be organized as a Cargo workspace with a shared library for configuration, logging, and test helpers; a server binary for the HTTP service; and an admin binary stub for database migrations.

## Technical Context

**Language/Version**: Rust (NEEDS CLARIFICATION: toolchain version, e.g., 1.75+)  
**Primary Dependencies**: dotenvy (env loading), log + tracing + tracing-subscriber (logging); (NEEDS CLARIFICATION: HTTP runtime — Axum/Tokio for future APIs vs minimal hyper for health only; choose Axum to align with standardization)  
**Storage**: PostgreSQL via SQLx + sqlx_migrations (admin stub only in this feature)  
**Testing**: cargo test; rstest; insta for snapshots; proptest for property tests; bats for shell scripts  
**Target Platform**: Linux/macOS servers, containerized runtime  
**Project Type**: Cargo workspace: shared library crate + server bin crate + admin bin crate (migration stub)  
**Performance Goals**: Health endpoint p95 ≤ 50 ms on localhost; startup to first healthy ≤ 2 s  
**Constraints**: No secrets in logs; redact sensitive values; graceful shutdown ≤ 2 s; 12‑Factor config precedence (env > .env > defaults)  
**Scale/Scope**: Single process, single endpoint (health) in this feature; prepares for future Axum/Tokio REST APIs

Open Questions (to resolve in Phase 0 research):
- Exact Rust toolchain version and MSRV policy (proposed: stable latest, MSRV = last stable‑2).
- Crate selection confirmation: Axum/Tokio vs minimal hyper for health only (proposed: Axum/Tokio for consistency with technical guide for REST APIs).
- Health payload shape (proposed: `{ "status": "ok" }`).
- Config keys and defaults (proposed: PORT=8080, HEALTH_PATH=/healthz, LOG_LEVEL=info).

## Constitution Check

Gate assessment based on constitution:
- Security-First: No secrets persisted or logged; config loader will redact sensitive keys (PASS with implementation notes).
- Test-First and Quality Gates: TDD with rstest/insta; unit + integration tests for health and config; linters in CI (PASS).
- Observability: Structured logs; metrics/traces earmarked for later; logs adequate for this feature (PASS for scope).
- Versioning/Change Control: Feature via JJ bookmark; plan/spec/tasks tracked; SemVer for crates (PASS).
- Simplicity: Minimal endpoints and crates to meet requirements; no premature optimization (PASS).

Re-check after Phase 1 to ensure contracts and quickstart reflect these gates.

## Project Structure

### Documentation (this feature)

```text
specs/002-setup-12fa-server/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
```

### Source Code (repository root)

```text
# Cargo workspace (to be scaffolded in implementation phase)
Cargo.toml                  # [workspace]
crates/
├── core/                   # Library crate: config, logging, test helpers
│   ├── Cargo.toml
│   └── src/
├── server/                 # Binary crate: HTTP server (health endpoint)
│   ├── Cargo.toml
│   └── src/
└── admin/                  # Binary crate: admin tasks (migrations stub)
    ├── Cargo.toml
    └── src/
```

**Structure Decision**: Adopt a Cargo workspace with three crates (core lib, server bin, admin bin). This enforces single responsibility, enables reuse of configuration/logging, and provides a home for test utilities.

## Complexity Tracking

No violations anticipated. If additional crates are requested, they will require approval per technical guide (“Do not introduce new crates without approval”).
