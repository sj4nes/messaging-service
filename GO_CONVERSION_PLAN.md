# Feature 011: Golang Conversion Plan (Spec Seed)

Status: Draft seed
Owner: TBD
Date: 2025-11-10
Scope: Conversion of messaging-service from Rust to Go with API parity and security posture preserved (or improved).

## 1) Overview & Goals

Convert the current Rust-based messaging-service to a Go implementation while preserving:
- API surface and JSON contracts (backwards compatibility for clients)
- Security posture aligned with OWASP controls (Feature 010 baseline)
- Observability (logs/metrics) and operational readiness (Docker, CI scanners)
- Comparable performance and resource usage targets

Non-goals (initial phase):
- Introducing new features beyond parity
- Multi-cloud deployment artifacts; reuse existing Docker/Compose where possible

## 2) User Stories (P1 first)

- US1 (P1): As a developer, I can run a Go binary/API exposing the same endpoints and JSON shapes as the Rust server so existing tests and clients continue to work unchanged.
- US2 (P1): As a security engineer, the Go service enforces the same security headers, rate limits, and auth/authorization rules as the Rust service.
- US3 (P1): As an operator, I can build and run the Go service via Docker Compose and see structured logs and Prometheus metrics, identical in names/labels where practical.
- US4 (P2): As a developer, all data access is type-safe and parameterized (sqlc) with the same schema and migrations; DB operations produce identical outcomes.
- US5 (P2): As a security engineer, CI scanners for Go (govulncheck, go-licenses, cyclonedx SBOM) run and gate merges.
- US6 (P3): As a product engineer, provider mocks (SMS/MMS, Email) function the same way under deterministic seeds.

## 3) Functional Requirements

- FR-API-Compat: Preserve routes, methods, status codes, and JSON schema of responses/requests.
- FR-Security-Headers: Apply CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, and HSTS (configurable) on all responses.
- FR-AuthZ: Enforce auth and authorization guards consistent with current rules; include session expiry and rate limiting of auth failures.
- FR-Validation: Validate inputs and parameterize all SQL (via sqlc generated code).
- FR-SSRF: Outbound allowlist enforcement for any HTTP egress initiated by the service.
- FR-Providers: Recreate provider mocks and registry behavior (routing per channel) with deterministic RNG seeds.
- FR-Observability: Emit structured logs (with redaction), Prometheus metrics mirroring current names/labels where feasible.

## 4) Non-Functional Requirements

- NFR-Performance: Match or better current p95 latencies. Authentication p95 < 500ms; typical API p95 < 200ms.
- NFR-Security: Maintain OWASP coverage established in Feature 010; pass headers, SSRF, and auth tests.
- NFR-Operability: Docker multi-stage builds; small images; health/metrics endpoints present.
- NFR-Compliance: SBOM generation, dependency/license scanning in CI; lock down version reproducibility via go.mod/go.sum; no plaintext secrets.

## 5) Architecture Mapping (Rust → Go)

- HTTP Router: Axum → chi (github.com/go-chi/chi) or Echo/Fiber. Recommend chi for middleware ergonomics and stdlib net/http alignment.
- Async/Concurrency: Tokio → goroutines + contexts; deadlines/timeouts via context.Context.
- DB Layer: SQLx (macros, offline) → sqlc (sqlc.dev) with pgx driver; keeps compile-time type safety for queries.
- Migrations: Keep existing SQL migrations; apply via golang-migrate/migrate CLI or embedded migrate library.
- Config: envconfig or viper for env + file; mirror existing env vars (PORT, HEALTH_PATH, security tunables, CSP, allowlists).
- Logging: tracing → uber-go/zap (structured) with redaction helpers.
- Metrics: current metrics → prometheus/client_golang; preserve metric names and labels for continuity.
- Rate Limiting: existing governor layer → golang.org/x/time/rate.
- Circuit Breaker: custom/CB → sony/gobreaker.
- Crypto: argon2id → golang.org/x/crypto/argon2; AES-256-GCM → crypto/aes + cipher; secrets from Vault via github.com/hashicorp/vault/api.
- Testing: Rust tests → Go testing with httptest; reproduce integration semantics; use testify for assertions if desired.
- Security Scanners: cargo-audit/deny → govulncheck, go-licenses; SBOM via cyclonedx-gomod; container scans continue with Trivy; ZAP baseline unchanged.

## 6) API & Compatibility Strategy

- Preserve all public endpoints and payload shapes; document any unavoidable differences.
- Maintain HTTP status codes and error structures.
- Keep health and metrics paths identical.
- Provide a “compat mode” flag if deviations are temporarily necessary during migration.

## 7) Data Model & Persistence

- Reuse the same PostgreSQL schema and migrations.
- Use sqlc for query generation; adopt pgx for driver.
- Ensure parameterization and avoid dynamic SQL string concatenation.
- Transaction boundaries mirror current behavior.

## 8) Security & Compliance (OWASP Alignment)

- Headers: CSP, XFO, XCTO, Referrer-Policy, HSTS (config toggles).
- Auth: session expiry, token/API key validation, per-route enforcement.
- SSRF: outbound allowlist; forbid RFC1918/private ranges and link-local unless explicitly allowed; enforce DNS resolution checks.
- Secrets: integrate Vault client; never log secrets; redact sensitive fields in logs.
- SBOM & Scanners: cyclonedx-gomod SBOM; govulncheck; go-licenses; Trivy scans for images; ZAP baseline for HTTP.

## 9) Observability

- Logging: zap with fields for request IDs, user, IP; redact PII; JSON output.
- Metrics: Prometheus metrics with existing counters/gauges/histograms; include provider labels.
- Tracing (optional): OpenTelemetry SDK for Go if needed later.

## 10) Phased Plan (High-Level)

- Phase 0: Skeleton & Config
  - Initialize Go module, router, config, security headers middleware, health/metrics.
- Phase 1: Data Access & Models
  - Port models and read paths; integrate sqlc and pgx; run queries read-only.
- Phase 2: Core Routes & Providers
  - Port message ingestion/webhooks and provider mocks; deterministic seeds.
- Phase 3: Security Middleware
  - Auth/authz guards, rate limits, circuit breaker, SSRF allowlist, redaction.
- Phase 4: CI & Scanners
  - Add govulncheck, go-licenses, cyclonedx-gomod; keep Trivy/ZAP.
- Phase 5: Parity Tests & Perf
  - Recreate integration tests with httptest; measure p95 latency; optimize.
- Phase 6: Cutover Readiness
  - Dual-run or toggle; compare metrics; document rollout/rollback.

## 11) Success Criteria

- SC-API-Parity: All endpoints respond with expected shapes/status; client integrations unchanged.
- SC-Security: Security headers present; auth/authorization/SSRF controls enforced; scanners pass with zero criticals/highs.
- SC-Perf: Meets p95 latency targets; no significant regression vs Rust.
- SC-Operability: Docker build succeeds; health/metrics work; logs structured with redaction.
- SC-Compliance: SBOM generated; license checks pass; dependency scans clean or actioned.

## 12) Risks & Mitigations

- R1: Performance regression → Benchmark and profile (pprof), tune GC; optimize hot paths.
- R2: Missing parity in edge cases → Build conformance tests from existing suite; add contract tests against JSON schemas.
- R3: SQL behavior differences → Validate transactions/isolation; align sqlc queries with SQLx semantics.
- R4: Security drift → Keep a security checklist mirroring Feature 010; add explicit tests for headers, auth, SSRF.
- R5: Operational surprises → Stage rollout behind a flag; run shadow traffic or canary.

## 13) Edge Cases

- Large payloads and DoS controls (body limits/timeouts).
- Unicode/segmentation nuances in snippets.
- Provider breaker behavior under sustained errors.
- Network partitions and retry/circuit states.
- Deterministic RNG behavior for provider mocks across restarts.

## 14) Acceptance & Verification

- Contract tests for API shapes and status codes.
- Security tests for headers, 401/403, SSRF allowlist, rate limits.
- DB parity checks for list endpoints and mutations.
- Perf smoke (k6) with budgets.
- CI gates: govulncheck, go-licenses, cyclonedx-gomod SBOM, Trivy, ZAP baseline.

## 15) Open Questions

- Router choice: chi vs Echo vs Fiber — defaulting to chi unless specific needs arise.
- Direct pgx vs database/sql: sqlc + pgx recommended for performance and ergonomics.
- Tracing (OpenTelemetry) inclusion in MVP or later phase.

---

This document is a seed spec/plan for Feature 011 (Golang conversion). If approved, I can expand it into tasks or a detailed execution plan and wire initial scaffolding.
