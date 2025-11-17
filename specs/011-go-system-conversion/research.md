# Research: Go System Conversion (Feature 011)

Date: 2025-11-10
Branch: 011-go-system-conversion
Spec: specs/011-go-system-conversion/spec.md
Plan: specs/011-go-system-conversion/plan.md

## Decisions & Rationale

### 1. Router Choice
- Decision: Use `chi` (github.com/go-chi/chi).
- Rationale: Lightweight, idiomatic net/http compatibility, flexible middleware chaining, minimal abstractions.
- Alternatives: Echo (more batteries; larger surface), Fiber (fast but non-net/http core), Gin (popular but less flexible for nested middleware).

### 2. Minimum Go Version
- Decision: Require Go 1.22+.
- Rationale: Access to language/runtime improvements, security fixes, and module proxy behavior; aligns with current ecosystem stability.
- Alternatives: 1.21 (LTS but fewer features), “latest” (risk of churn).

### 3. DB Layer (sqlc + pgx)
- Decision: Use sqlc to generate type-safe code from existing SQL; pgx as driver.
- Rationale: Closest analogue to SQLx compile-time safety; avoids reflection; clear query ownership.
- Alternatives: GORM (less predictable performance), Ent (schema-first requiring translation), plain database/sql (runtime errors only).

### 4. Migrations Strategy
- Decision: Reuse existing SQL migrations via `golang-migrate` CLI invoked in startup/CI pipeline.
- Rationale: Avoid duplication; maintain single source of truth; easy integration in container entrypoint.
- Alternatives: Embed migrations in Go binary (increase size) or re-author using a Go DSL.

### 5. Secrets Handling (Dev & Prod)
- Decision: Abstract secrets through interface; production uses Vault API client; development uses file/ENV stubs explicitly flagged and redacted in logs.
- Rationale: Keeps constitution compliance while allowing rapid local development.
- Alternatives: Direct ENV only (violates secrets hygiene), mock Vault server (additional complexity).

### 6. Logging
- Decision: Use `zap` with a structured logger + custom redaction function invoked for sensitive fields.
- Rationale: Performance, structured output, ecosystem maturity.
- Alternatives: zerolog (similar benefits; chosen zap due to broader familiarity), logrus (slower, older design).

### 7. Metrics
- Decision: Use `prometheus/client_golang` with metric name mapping where needed; no strict legacy name parity required.
- Rationale: Standard integration path; existing tooling expects Prometheus exposition format.
- Alternatives: OpenTelemetry metrics (adds complexity early), custom aggregation.

### 8. Rate Limiting
- Decision: `golang.org/x/time/rate` per key (IP, sender) with lightweight wrappers; maintain configuration parity.
- Rationale: Minimal dependencies; standard library-adjacent.
- Alternatives: Redis-based global limiter (overkill for parity baseline).

### 9. Circuit Breaker
- Decision: `sony/gobreaker` for provider and global breakers.
- Rationale: Well-tested, configurable states similar to existing Rust implementation.
- Alternatives: Roll custom (reinvent logic), resilience libraries (heavier).

### 10. Provider Determinism
- Decision: Seed `math/rand` sources per provider with configured deterministic seed values; separate RNG instances per channel.
- Rationale: Parity with Rust seeded behavior for test repeatability.
- Alternatives: crypto/rand (non-deterministic), global seed only (reduces isolation).

### 11. SSRF Protection
- Decision: Implement allowlist validator performing hostname resolution, blocking RFC1918 and link-local ranges unless explicitly configured.
- Rationale: Matches security expectations and OWASP guidance.
- Alternatives: Simple domain string match (insufficient), proxy-based filtering (future enhancement).

### 12. Headers (Security)
- Decision: Apply CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, HSTS via middleware; configure via env.
- Rationale: Straightforward parity and external configurability.
- Alternatives: Hardcoded policies (less flexible), dynamic DB-driven headers (scope creep).

### 13. Testing Strategy
- Decision: Separate contract (API shape), integration (DB interactions), security (auth/headers/SSRF/rate limits), unit (helpers, logic). Use table-driven tests.
- Rationale: Clear layering; mirrors existing Rust test taxonomy.
- Alternatives: Monolithic integration tests (slower feedback), BDD frameworks (extra overhead).

### 14. Performance Verification
- Decision: k6 smoke tests run against critical endpoints to assert latency budgets pre-merge.
- Rationale: Light automation ensures early detection of regressions.
- Alternatives: Full load tests early (time/infra heavy).

### 15. SBOM & Scanning
- Decision: Use cyclonedx-gomod for SBOM, govulncheck for vulnerability scanning, go-licenses for license compliance; integrate into CI gating.
- Rationale: Constitution mandates SBOM and license verification; standard tools.
- Alternatives: Manual review (not scalable), custom scripts (maintenance burden).

### 16. Cutover Mechanism
- Decision: Direct replacement accepted; provide rollback script (rename binary or re-point compose service) and readiness checklist.
- Rationale: No production traffic; simplifies migration overhead.
- Alternatives: Canary/shadow (unnecessary complexity).

## Alternatives Summary (Condensed)
- Router: chi > echo/fiber due to simplicity.
- ORM: sqlc+pgx > GORM/Ent for compile-time safety.
- Secrets: Vault API + stub > plain env.
- Logging: zap > zerolog due to familiarity.
- Metrics: Prometheus > OTEL (initial complexity).

## Open Questions (Resolved)
None remaining.

## Risks
| Risk | Impact | Mitigation |
|------|--------|-----------|
| Performance regression | Slower responses | Profiling (pprof), tune goroutines, allocate pools |
| Missing edge parity | Behavior differences | Contract + integration parity test suite |
| Secrets misconfiguration | Security exposure | Interface abstraction + forbidding plaintext in code reviews/CI checks |
| SSRF bypass via DNS tricks | Internal resource access | Canonicalize IPs post-resolution; block private CIDRs |
| Metric drift confusion | Ops dashboards break | Provide mapping doc for any renamed key metrics |

## Summary
All unknowns resolved; design can proceed.
