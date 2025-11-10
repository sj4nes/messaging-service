# Implementation Plan: Go System Conversion

**Branch**: `011-go-system-conversion` | **Date**: 2025-11-10 | **Spec**: `specs/011-go-system-conversion/spec.md`
**Input**: Feature specification prepared under `specs/011-go-system-conversion/spec.md`

## Summary

Convert the current Rust messaging-service to a Go implementation while preserving API, security, and operability parity. Direct replacement is acceptable (no rollout needed). Metric names need not be identical; tests will assert required signals. Technical approach (pending research) targets a standard Go HTTP stack with type-safe SQL access and equivalent security/observability controls.

## Technical Context

**Language/Version**: Go (NEEDS CLARIFICATION: minimum version; default 1.22+)  
**Primary Dependencies**: Router (NEEDS CLARIFICATION: chi vs echo vs fiber), sqlc + pgx for DB, logging (zap), metrics (prometheus client)  
**Storage**: PostgreSQL (reuse existing schema/migrations)  
**Testing**: Go testing + httptest; contract/integration tests; security tests; perf smoke (k6)  
**Target Platform**: Linux containers; macOS dev; Compose orchestration  
**Project Type**: Backend API service  
**Performance Goals**: p95 typical API ≤ 200ms; p95 auth ≤ 500ms  
**Constraints**: API parity; security parity (headers, auth/rl, SSRF); direct replacement acceptable  
**Scale/Scope**: Parity with existing feature set (messages/conversations/providers)

Unknowns (to resolve in research):
- Router choice and rationale (chi vs echo vs fiber)
- Minimum Go version policy
- sqlc + pgx specifics (migrations flow and local dev ergonomics)
- Secrets handling for dev (Vault stubs vs env files) without violating constitution

## Constitution Check

GATE PRE-DESIGN: Tentative PASS with conditions
- Security-First: Plan includes auth/authorization, headers (incl. HSTS), SSRF allowlist, rate limits.  
- Secrets Hygiene: Must avoid plaintext secrets; plan to integrate Vault client or dev-safe fallback with strict redaction.  
- Test-First and Quality Gates: Require unit/integration + contract tests; CI scanners for dependencies/licenses/SBOM; ZAP/Trivy to continue.  
- Observability: Structured logs with redaction; metrics exposed; alertability preserved.  
- Compliance: SBOM (cyclonedx-gomod), license policy (go-licenses), dependency scanner (govulncheck) must be added.

Decision: Proceed to Phase 0 (research) to resolve unknowns and confirm gates.

## Project Structure

### Documentation (this feature)

```text
specs/011-go-system-conversion/
├── plan.md         # This file
├── research.md     # Phase 0 output (decisions & rationale)
├── data-model.md   # Phase 1 entity & validation design
├── quickstart.md   # Phase 1 operational + verification guide
└── contracts/      # Phase 1 API contracts (OpenAPI)
```

### Source Code (impact overview)

```text
go/                 # New Go module root (to be introduced later)
  cmd/server/       # Main entrypoint
  internal/         # Packages (router, middleware, db, providers, metrics, logging)
  api/              # Handlers, request/response models
tests/              # Go tests (contract/integration/unit)
```

**Structure Decision**: Introduce a new `go/` module tree alongside existing Rust code for parallel development and parity testing.

## Complexity Tracking

No constitution violations expected if secrets are properly handled and CI gates added for Go dependencies and SBOM.
