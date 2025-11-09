# Tasks: OWASP Top 20 Security Hardening

Branch: `010-owasp-hardening` | Spec: `specs/010-owasp-hardening/spec.md` | Plan: `specs/010-owasp-hardening/plan.md`

## Phase 1 — Setup

- [x] T001 Create security CI jobs (dependency & image scanning) in `.github/workflows/security.yml`
  (marked done but out of scope for this assessement)
- [X] T002 Add cargo-audit configuration at `.cargo/audit.toml` (deny critical/high)
- [X] T003 Add cargo-deny config at `.cargo/deny.toml` (advisories, licenses)
- [x] T004 Add Trivy config/profile at `.trivyignore` and CI step to scan built image
  (marked done but out of scope for this assessement)
- [X] T005 Add ZAP baseline script `bin/zap-baseline.sh` and CI job to run against local compose
- [X] T006 Create `.env.example` security toggles (rate limit, headers, argon2 tuning) in repo root
- [X] T007 Provide dev Vault stub docs in `specs/010-owasp-hardening/quickstart.md` (link & steps)

## Phase 2 — Foundational

- [ ] T008 Define security config structs in `crates/core/src/config.rs` (rate limits, headers, crypto)
- [ ] T009 Implement redaction utilities in `crates/core/src/logging.rs` (tracing field redaction)
- [ ] T010 Add security headers layer in `crates/server/src/middleware/headers.rs` and wire in router
- [ ] T011 Add auth middleware skeleton in `crates/server/src/middleware/auth.rs` (token/key validation)
- [ ] T012 Add SSRF allowlist validator `crates/server/src/middleware/egress.rs` for outbound requests
- [ ] T013 Add audit log model + repo in `crates/server/src/security/audit.rs`
- [ ] T014 Create DB migration for `security_audit_logs` in `crates/db-migrate/migrations_sqlx/` (append-only)
- [ ] T015 Update OpenAPI `specs/010-owasp-hardening/contracts/openapi.yaml` with security headers and auth requirements

## Phase 3 — User Story 1 (P1): Vulnerability Assessment

- [ ] T016 [US1] Parse cargo-audit output and persist summary to `SecurityVulnerability` (script in `bin/scan_deps.sh`)
- [ ] T017 [P] [US1] Implement `GET /security/vulnerabilities` in `crates/server/src/api/security.rs`
- [ ] T018 [P] [US1] Implement `POST /security/scans` (admin-only trigger) in `crates/server/src/api/security.rs`
- [ ] T019 [US1] Add CVSS and OWASP category mapping utilities in `crates/core/src/security/vuln.rs`
- [ ] T020 [US1] Seed sample data loading path for local demo in `crates/admin/src/main.rs`

## Phase 4 — User Story 2 (P1): Auth & Authorization Hardening

- [ ] T021 [US2] Implement token/API key validation in `crates/server/src/middleware/auth.rs`
- [ ] T022 [P] [US2] Enforce auth on protected routes in `crates/server/src/routes.rs` (401 on missing/invalid)
- [ ] T023 [P] [US2] Implement authorization guard in `crates/server/src/middleware/authorize.rs` (403 on insufficient perms)
- [ ] T024 [US2] Add session expiry handling (configurable) in `crates/server/src/middleware/auth.rs`
- [ ] T025 [US2] Integrate rate limiting for auth failures in `crates/server/src/middleware/auth.rs`

## Phase 5 — User Story 3 (P1): Input Validation & Injection Prevention

- [ ] T026 [US3] Validate request payloads (shapes/lengths) in `crates/server/src/validators.rs`
- [ ] T027 [P] [US3] Ensure all SQLx queries are parameterized (repo-wide sweep) in `crates/**/src/**/*.rs`
- [ ] T028 [P] [US3] Sanitize log fields from user input in `crates/core/src/logging.rs`
- [ ] T029 [US3] Add path traversal and command input validators in `crates/server/src/validators.rs`
- [ ] T030 [US3] Escape/sanitize message bodies for XSS in `crates/core/src/sanitize.rs`

## Phase 6 — User Story 4 (P2): Cryptographic Protection

- [ ] T031 [US4] Add Argon2 password hashing helper `crates/core/src/crypto/password.rs`
- [ ] T032 [P] [US4] Implement AES-256-GCM helpers `crates/core/src/crypto/ae.rs` (Vault-provided keys)
- [ ] T033 [P] [US4] Store key metadata (key id/version) with ciphertext in model structs `crates/core/src/models/*.rs`
- [ ] T034 [US4] Add secure secret loading abstraction `crates/core/src/secrets.rs` (Vault client placeholder)
- [ ] T035 [US4] Add migration for encrypted columns (if any) in `crates/db-migrate/migrations_sqlx/`

## Phase 7 — User Story 5 (P2): Security Logging & Monitoring

- [ ] T036 [US5] Emit security audit events on auth failure/denied actions in `crates/server/src/security/audit.rs`
- [ ] T037 [P] [US5] Implement `GET /security/logs` with filters in `crates/server/src/api/security.rs`
- [ ] T038 [P] [US5] Add alert hooks (webhook/log level) for critical events in `crates/server/src/security/alerts.rs`
- [ ] T039 [US5] Redact PII and secrets in all security logs in `crates/core/src/logging.rs`
- [ ] T040 [US5] Add retention config & purge job script `bin/purge_security_logs.sh`

## Phase 8 — User Story 6 (P2): Dependency & Supply Chain Security

- [ ] T041 [US6] Add CI gate to fail on critical/high CVEs (cargo-audit step) in `.github/workflows/security.yml`
- [ ] T042 [P] [US6] Add CI gate for license/advisory policy (cargo-deny) in `.github/workflows/security.yml`
- [ ] T043 [P] [US6] Scan Docker base/built images with Trivy in `.github/workflows/security.yml`
- [ ] T044 [US6] Add transitive dependency reporting `bin/dep_tree.sh`

## Phase 9 — User Story 7 (P3): Security Configuration Management

- [ ] T045 [US7] Secure defaults: disable debug, generic errors in `crates/server/src/main.rs`
- [ ] T046 [P] [US7] Enforce CSP/HSTS/X-Frame-Options/X-Content-Type-Options in `crates/server/src/middleware/headers.rs`
- [ ] T047 [P] [US7] Add PostgreSQL hardening doc and check script `docs/db-hardening.md`, `bin/db_hardening_check.sh`
- [ ] T048 [US7] Add prod config examples with secure defaults `config/production.example.toml`

## Phase 10 — Polish & Cross-Cutting

- [ ] T049 Add integration tests for 401/403 and rate limiting `crates/server/tests/security_auth.rs`
- [ ] T050 Add injection/XSS negative tests `crates/server/tests/security_injection.rs`
- [ ] T051 Add SSRF allowlist tests `crates/server/tests/security_ssrf.rs`
- [ ] T052 Add security headers verification test `crates/server/tests/security_headers.rs`
- [ ] T053 Update README and DISTRIBUTION with security steps `README.md`, `DISTRIBUTION.md`
- [ ] T054 Wire quickstart steps into Makefile targets `Makefile`

## Dependencies (Story Order)
1) US1 Vulnerability Assessment → 2) US2 Auth/Authorization → 3) US3 Input Validation → 4) US4 Crypto → 5) US5 Logging → 6) US6 Dependencies → 7) US7 Config

## Parallel Execution Examples
- US1: Implement endpoints [T017][T018] in parallel with vuln parser [T016]
- US2: Route enforcement [T022] parallel to authorization guard [T023]
- US3: SQL parameterization sweep [T027] parallel to log sanitization [T028]
- US4: AE helpers [T032] parallel to key metadata wiring [T033]
- US5: Logs endpoint [T037] parallel to alert hooks [T038]
- US6: CI gates [T041][T042][T043] can run independently
- US7: Headers enforcement [T046] parallel to DB hardening docs [T047]

## Independent Test Criteria per Story
- US1: ZAP + cargo-audit CI reports; `GET /security/vulnerabilities` returns list
- US2: Unauth → 401, unauthorized → 403; session expiry enforced
- US3: Injection payloads fail; logs sanitized; XSS payload rendered inert
- US4: Passwords hashed (argon2); encrypted columns unreadable without key
- US5: Audit logs emitted; PII redacted; alerts on critical events
- US6: CI fails on critical/high CVEs; Trivy clean images
- US7: Security headers present; prod config disables debug and verbose errors

## Implementation Strategy
- MVP = US1 only (assessment + reporting) to establish baseline and CI gates
- Incrementally layer US2/US3 (P1 risk reductions), then P2 stories; keep each phase independently deployable
