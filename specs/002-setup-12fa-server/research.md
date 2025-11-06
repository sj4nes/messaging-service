# Phase 0 Research — Setup 12‑Factor Server Bootstrap

**Date**: 2025-11-05  
**Branch**: 002-setup-12fa-server

## Decisions and Rationale

1) Rust Toolchain and MSRV
- Decision: Use latest stable Rust for development; set MSRV to stable−2 for CI compatibility.
- Rationale: Balances modern features with CI stability; aligns with common ecosystem practice.
- Alternatives: Pin exact version (less flexibility), nightly (unnecessary for scope).

2) HTTP Runtime (Axum/Tokio vs minimal hyper)
- Decision: Use Axum over Tokio/hyper stack for routing, even if only a health route exists now.
- Rationale: Technical guide standardizes Axum/Tokio for REST APIs; consistency reduces future refactors.
- Alternatives: hyper direct (lower-level, more boilerplate) or tiny_http (not aligned with future needs).

3) Config Loading and Precedence
- Decision: dotenvy for .env loading with precedence env > .env > defaults.
- Rationale: Aligns 12‑Factor; dotenvy matches technical guide.
- Alternatives: envy/config crates (additional mapping/indirection not necessary initially).

4) Logging
- Decision: tracing + tracing-subscriber with log compatibility; structured JSON optional behind a feature flag later.
- Rationale: Technical guide mandates log/tracing; structured logs ease ops.
- Alternatives: env_logger only (insufficient for structured contexts at scale).

5) Health Endpoint Payload
- Decision: 200 OK with `{ "status": "ok" }` body, content-type application/json.
- Rationale: Minimal liveness; easy to extend with version/build later.
- Alternatives: Empty 204 (less informative), text/plain (less standardized).

6) Admin Migration Stub
- Decision: Add admin binary crate with a stub command `migrate` using sqlx_migrations (no-op for now).
- Rationale: Prepares for DB work; aligns with technical guide (PostgreSQL + SQLx).
- Alternatives: Defer entirely (missed opportunity to wire structure early).

## Resolved Unknowns

- Rust toolchain: latest stable; MSRV stable−2.
- HTTP: Axum/Tokio.
- Config keys: PORT (default 8080), HEALTH_PATH (default /healthz), LOG_LEVEL (default info).
- Health response shape: `{ "status": "ok" }`.
- Logging: tracing with redaction for sensitive keys; JSON support later.

## Implications

- Workspace must include Axum/Tokio deps; future API routes can be added incrementally.
- Test strategy: rstest for unit tests, integration tests for server start/health response; insta snapshots optional for response schema.
- Security: No secrets logged; redact known keys; no auth on health route by design.

## References

- Technical Guide: logging (log, tracing, tracing-subscriber), dotenvy, SQLx/sqlx_migrations, Axum/Tokio.
- 12‑Factor App methodology.