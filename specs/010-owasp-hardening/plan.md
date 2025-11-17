# Implementation Plan: OWASP Top 20 Security Hardening

**Branch**: `010-owasp-hardening` | **Date**: 2025-11-08 | **Spec**: `specs/010-owasp-hardening/spec.md`
**Input**: Feature specification prepared under `specs/010-owasp-hardening/spec.md`

## Summary

Implement comprehensive hardening across authentication, authorization, cryptography, input validation, dependency management, logging/monitoring, and configuration to eliminate OWASP Top 10 risks (mapped in spec) and achieve measurable success criteria (zero critical vulns, full endpoint protection, <48h MTTR). Technical approach introduces security scanning (cargo-audit, cargo-deny, Trivy), standardized password hashing (argon2), rate limiting consolidation (existing metrics integration), secret management abstraction, and structured security logging with redaction.

## Technical Context

**Language/Version**: Rust (stable, 1.83+ confirmed via rust:slim image)  
**Primary Dependencies**: Axum (HTTP), Tokio (async), SQLx (PostgreSQL, offline mode), Serde (serialization), Tracing (observability), governor (rate limiting), argon2 (password hashing), secrecy (secret wrapper)  
**Storage**: PostgreSQL (messages, users, conversations); HashiCorp Vault for secrets and encryption keys  
**Testing**: cargo test (unit + integration), plus security scanners: cargo-audit (CVE), cargo-deny (licenses + advisories), Trivy (container image), OWASP ZAP baseline for DAST  
**Target Platform**: Linux containers (Docker), macOS dev environment; TLS termination at ingress/proxy (service internal over trusted network)  
**Project Type**: Multi-crate Rust workspace (`crates/core`, `crates/server`, `crates/admin`, `crates/db-migrate`)  
**Performance Goals**: Maintain existing latency goals (<200ms p95 for typical API calls) while adding crypto overhead; authentication ops p95 <500ms (spec)  
**Constraints**: SQLx offline builds (no live DB at build time), minimal image size (~<200MB), secure defaults (no debug in prod), secrets not in env plain text (constitution)  
**Scale/Scope**: Initial hardening for current feature set (conversations/messages) with extensibility for future providers; rate limits configurable with documented defaults

### Derived Components
| Area | Planned Addition | Notes |
|------|------------------|-------|
| Auth | Session/token validation middleware | Reuses Axum layers; pluggable for future OAuth/OIDC |
| Crypto | Argon2 password hashing, AES-256-GCM for sensitive columns | Key storage needs design (Vault/KMS) |
| Logging | Security audit log struct + redaction layer | Integrate with Tracing subscriber |
| Dependency Security | CI jobs for cargo-audit, cargo-deny | Fail on critical/high issues |
| SSRF Protection | Outbound request validator | Allowlist-based; minimal initial scope |
| Config Hardening | Security header middleware | CSP/HSTS/X-Frame-Options, configurable |

## Constitution Check (Post-Research Gate)

| Principle | Compliance | Notes |
|-----------|-----------|-------|
| Security-First | Pass | Secrets via Vault; input validation, encryption, authZ planned and tested |
| Test-First | Pass | CI to include SAST/DAST, dependency scans; security tests planned |
| Observability | Pass | Audit logging schema and redaction defined; aggregation optional |
| Versioning & Change Control | Pass | Feature isolated to branch `010-owasp-hardening` |
| Simplicity & SRP | Pass | Additions scoped, no new crate proliferation yet |

| Safety & Compliance Requirement | Status | Clarification |
|---------------------------------|--------|--------------|
| Input validation all external inputs | Pass | Enumerate and test API + webhook inputs; reject malformed |
| Secrets external manager | Pass | Vault selected; no plaintext secrets |
| TLS & data encryption | Pass | TLS at ingress; AES-256-GCM column encryption with Vault keys |
| SAST/DAST in CI | Pass | cargo-audit/deny + Trivy + ZAP baseline configured in plan |
| Rate limits | Pass | Defaults set; configurable with monitoring |

GATE RESULT: PASS — proceed to Phase 1 design & contracts.

## Project Structure

### Documentation (this feature)

```text
specs/010-owasp-hardening/
├── plan.md         # This file
├── research.md     # Phase 0 output (decisions & rationale)
├── data-model.md   # Phase 1 entity & validation design
├── quickstart.md   # Phase 1 operational + verification guide
├── contracts/      # Phase 1 API contracts (OpenAPI)
└── spec.md         # Existing specification
```

### Source Code (impact overview)

```text
crates/
  core/            # Config, logging redaction extensions (new security config structs)
  server/          # Axum routes, middleware (auth, headers, rate limit)
  admin/           # Potential vulnerability/report endpoints (read-only)
  db-migrate/      # Future migrations for security tables (audit log, user auth)
```

**Structure Decision**: Retain existing multi-crate workspace; extend `core` for security configuration and redaction utilities; extend `server` for middleware/endpoints; avoid new crates to preserve simplicity.

## Complexity Tracking

No constitution violations requiring justification at this stage. If secrets manager integration introduces external SDK complexity, will revisit.

## Phase 0: Research Plan

### Unknowns Extracted
1. Password hashing algorithm choice (argon2 vs bcrypt)  
2. Secret manager/KMS selection (HashiCorp Vault vs cloud provider)  
3. TLS termination location (service vs ingress proxy)  
4. Data at rest encryption approach (application-level column encryption vs pgcrypto vs external KMS envelope)  
5. Expected concurrent user volume (for rate limit sizing)  
6. Outbound request allowlist source (static config vs dynamic DB)  
7. DAST tooling integration (OWASP ZAP vs custom script)  
8. Multi-factor authentication scope (in-scope now or future)  
9. Key rotation cadence (manual quarterly vs automated)  
10. Audit log storage medium (PostgreSQL table vs append-only file vs external log service)

### Research Tasks
For each unknown, produce: Decision, Rationale, Alternatives.

Security Best Practices Tasks:
- Best practices: Argon2 parameters in high-throughput Rust services.
- Best practices: Vault vs AWS Secrets Manager performance/latency tradeoffs.
- Best practices: Implementing structured security audit logging with Tracing.
- Best practices: Avoiding SQL injection with SQLx patterns (double-check parameterization).
- Best practices: Container image scanning frequency and Trivy integration.

Phase 0 Output: `research.md` consolidating all decisions; unresolved items must NOT remain.

## Phase 1: Design & Contracts Plan

### Data Model Targets
Entities from spec: SecurityVulnerability, SecurityAuditLog, AuthenticationAttempt, DependencyManifest. Extend with KeyRotationEvent (derived need) and RateLimitSnapshot (for monitoring). Include validation (e.g., CVSS range 0.0–10.0, timestamp monotonicity).

### API Contracts
Add endpoints (tag: security):
- GET /security/vulnerabilities
- POST /security/scans
- GET /security/dependencies
- GET /security/logs (filtered)
- POST /auth/login
- POST /auth/logout
- POST /auth/rotate-keys
- GET /auth/rate-limit/status
- GET /security/health (aggregated security posture)

OpenAPI spec stored at `contracts/openapi.yaml` with schemas for entities and standard error model.

### Quickstart
Operational guide to enable scanners, run security tests, interpret results, rotate keys, verify headers.

### Agent Context Update
Run `.specify/scripts/bash/update-agent-context.sh copilot` after design artifacts to append security technology context.

## Phase 2 (Out of Scope Here)
Will generate tasks for implementation (separate `/speckit.tasks`).

## Post-Design Constitution Re-Check (Planned)
To be updated after artifacts created; expect Security-First moves to PASS once secret manager and crypto decisions finalized.

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|-----------|
| Secret manager adds latency | Slower auth/encryption operations | Caching secret metadata; benchmark before adoption |
| Overly strict rate limits | User experience degradation | Iterative tuning with observed metrics |
| Log volume surge | Storage cost & performance issues | Sampling non-security debug logs; separate audit log retention class |
| Encryption overhead | Increased CPU usage | Benchmark AES-GCM vs ChaCha20-Poly1305; adjust hardware sizing |
| Toolchain failures in CI | Blocked deploys | Allow temporary override with security officer approval (documented) |

## Acceptance Alignment
Each success criterion in spec mapped to artifact/test:
| Success Criterion | Artifact/Test |
|-------------------|--------------|
| SC-001 zero critical vulns | cargo-audit + Trivy reports CI gating |
| SC-002 OWASP categories covered | OpenAPI + middleware + tests summary doc |
| SC-003 endpoint auth protection | Integration test suite `auth_protection.rs` |
| SC-004 no plaintext sensitive data | DB inspection script + migration ensures hashing/encryption |
| SC-005 parameterized SQL only | Static analysis (grep) + sqlx compile checks |
| SC-006 security logging coverage | Log schema tests + event emission tests |
| SC-007 dependency scanning alerts | CI config (cargo-audit, cargo-deny) |
| SC-008 MTTR <48h | Incident tracking playbook + metrics dashboard |
| SC-009 100% security tests pass | CI test summary step |
| SC-010 90-day no incidents | Post-deploy monitoring review entries |

## Implementation Notes
Focus minimal invasive changes early: add audit log table & middleware, integrate scanning in CI, then progressive hardening (headers, rate limit tuning, encryption). Defer MFA if out-of-scope until clarified.

---
Plan prepared; proceed to Phase 0 research generation.
