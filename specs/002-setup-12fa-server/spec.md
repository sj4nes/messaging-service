# Feature Specification: Setup 12‑Factor Server Bootstrap

**Feature Branch**: `002-setup-12fa-server`  
**Created**: 2025-11-05  
**Status**: Draft  
**Input**: User description: "We're now setting up a Rust-based 12-factor application. Refer to the technical guide for pre-ordained stack decisions. Our first setup will rough in a server that reads a .env file, starts a health check endpoint on a configured port, and emits logging events."

## User Scenarios & Testing (mandatory)

### User Story 1 — Start service and verify health (Priority: P1)

An operator starts the service in a local or container environment with configuration provided via environment variables and/or an .env file. The service loads configuration, binds to the configured port, and exposes a health endpoint that returns a successful response.

Why this priority: This is the minimal viable slice to validate the process can boot and be monitored, enabling deployment and integration in automation pipelines.

Independent Test: Start the process with a specified port, then perform an HTTP request to the health endpoint and assert a 200 response with expected payload; confirm startup and request logs are emitted.

Acceptance Scenarios:

1. Given PORT is set and .env is present, When the service starts, Then it binds to PORT and the health endpoint returns 200.
2. Given no .env but required variables are present in the environment, When the service starts, Then health returns 200 and logs indicate configuration source precedence.

---

### User Story 2 — Configuration precedence and defaults (Priority: P2)

An operator can supply settings via environment variables with an optional .env file. Environment variables take precedence. Reasonable defaults exist for optional values.

Why this priority: Aligns with 12‑Factor practices and reduces friction across environments (dev, CI, prod).

Independent Test: Provide conflicting values in env and .env and verify env wins; omit optional values and verify defaults are in effect.

Acceptance Scenarios:

1. Given LOG_LEVEL is set in both env and .env, When service starts, Then the env value is used and logged.
2. Given no LOG_LEVEL is set anywhere, When service starts, Then a default value is used and logged.

---

### User Story 3 — Graceful shutdown and error handling (Priority: P3)

The process handles common lifecycle events and failure modes: it exits with a clear error when the port is already in use, and it terminates gracefully on signals.

Why this priority: Improves operability and reliability in orchestration environments.

Independent Test: Start a dummy server to occupy the port and verify startup fails with non‑zero exit and a clear message; send SIGINT/SIGTERM and verify the process shuts down cleanly.

Acceptance Scenarios:

1. Given the configured port is already bound by another process, When the service starts, Then it exits non‑zero and logs a clear bind error.
2. Given the process is running, When it receives SIGTERM, Then it stops accepting requests, completes in‑flight work if any, logs shutdown, and exits within a bounded time.

### Edge Cases

- Invalid port value (non‑numeric or out of range) → fail fast with clear configuration error.
- Malformed .env entries (e.g., missing `KEY=VALUE` structure) → skip invalid lines and log warnings, or fail fast if a required key is malformed.
- Health endpoint path overridden to an empty or invalid value → fall back to default path.
- Slow or blocked startup (e.g., port bind delay) → startup timeout results in non‑zero exit and clear message.

## Requirements (mandatory)

### Functional Requirements

- FR‑001: The service MUST read configuration from process environment variables with optional overrides from a co‑located .env file.
- FR‑002: Environment variables MUST take precedence over .env values when both are present.
- FR‑003: The service MUST expose an HTTP health endpoint on a configurable port and path.
- FR‑004: The health endpoint MUST return a successful status (e.g., 200) and a minimal payload indicating liveness.
- FR‑005: The service MUST emit structured log events at minimum for startup, bound address/port, health request handling, and shutdown.
- FR‑006: On invalid configuration (e.g., missing required variables, invalid port), the service MUST exit with a non‑zero status and a clear error message.
- FR‑007: On bind failure due to port already in use, the service MUST exit non‑zero and log the cause.
- FR‑008: The service MUST support graceful shutdown on common termination signals (e.g., SIGINT, SIGTERM) within a bounded time.
- FR‑009: Reasonable defaults MUST exist for optional settings (e.g., log level, health path), and defaults MUST be documented in startup logs.
- FR‑010: The health endpoint MUST be accessible without authentication for readiness/liveness probing.

Assumptions

- Default port: 8080 if PORT not provided.
- Default health path: `/healthz` if not provided.
- Default log level: "info" if not provided.
- If both .env and environment are present, environment wins per 12‑Factor practices.

### Key Entities

- Runtime Configuration: keys such as `PORT`, `HEALTH_PATH`, `LOG_LEVEL`, plus room for future keys; includes precedence rules and defaults.
- Health Probe: a minimal response structure indicating liveness (e.g., `{ "status": "ok" }`) and optionally a timestamp.

## Success Criteria (mandatory)

### Measurable Outcomes

- SC‑001: From process start to first successful health check is ≤ 2 seconds on a typical developer machine.
- SC‑002: Health endpoint responds with success in ≤ 50 ms (95th percentile) on localhost under idle conditions.
- SC‑003: Startup logs include resolved port, health path, and configuration source precedence; shutdown logs are emitted on signal within 2 seconds.
- SC‑004: Misconfiguration or bind failure results in non‑zero exit and a human‑readable error message; no partial/daemonized processes remain.
- SC‑005: Health endpoint remains available and stable across restarts (two consecutive restarts without error within 60 seconds).
