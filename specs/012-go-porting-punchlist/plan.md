# Implementation Plan: Go Porting Punchlist

**Branch**: `012-go-porting-punchlist` | **Date**: 2025-11-12 | **Spec**: `specs/012-go-porting-punchlist/spec.md`
**Input**: Feature specification from `specs/012-go-porting-punchlist/spec.md`

**Note**: Generated via `/speckit.plan` workflow.

## Summary

Deliver a structured parity audit (gap inventory, remediation task mapping, closure validation) between Rust reference messaging service and Go port. Technical approach: analyze test harness + runtime behaviors; normalize list endpoint responses; verify metrics; produce artifacts (gap list, tasks, closure report) without introducing new functional scope beyond parity.

## Technical Context

**Language/Version**: Go 1.22+ (port) & Rust 1.83 (reference) – audit spans both, deliverables in Go docs/specs.  
**Primary Dependencies**: Go: chi (HTTP), pgx/sqlc (DB), zap (logging), Prometheus client (metrics); Rust: Axum, SQLx, Tracing (reference only).  
**Storage**: PostgreSQL primary (persistence), in-memory fallback (Go) for empty/failed cases; seed logic ensures baseline data.  
**Testing**: Existing contract/API tests (`tests/http/tests.json`) + added parity assertions; toolchains: `go test` (if present) & existing harness script; Rust test behavior reference only.  
**Target Platform**: Linux containers + macOS dev environment.  
**Project Type**: Multi-crate/multi-module back-end service with spec-driven planning.  
**Performance Goals**: Parity feature itself non-perf critical; must not materially degrade existing request latency (<200ms p95 baseline assumed).  
**Constraints**: No schema changes beyond seed normalization; avoid introducing new external dependencies; maintain security/observability gates.  
**Scale/Scope**: Limited to inventory + remediation planning; does not expand functional footprint or add high-scale processing.

Unknowns requiring confirmation: NONE (spec resolved). No NEEDS CLARIFICATION markers.

## Constitution Check

GATE Pre-Design Evaluation:
- Security-First: Parity audit reads existing code; no new external inputs introduced. PASS
- Test-First: Will add tests for list normalization & metrics parity before implementing changes. PASS (commit to TDD for new assertions)
- Observability: Metrics review ensures existing counters documented; may add worker_processed exposure consistency. PASS
- Versioning & Change Control: Documentation-only + minor API response shape normalization (empty array). Backwards compatible (clients expecting array). PASS
- Simplicity: Introduces no new runtime subsystems (worker deferred). PASS

No violations; Complexity Tracking section not required at this time.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
specs/012-go-porting-punchlist/
  spec.md
  plan.md
  research.md            (Phase 0)
  data-model.md          (Phase 1)
  quickstart.md          (Phase 1)
  contracts/             (Phase 1)
  checklists/requirements.md

go/ (Go port source)
  api/
  internal/db/
  internal/metrics/
  cmd/server/
crates/ (Rust reference implementation)
tests/http/tests.json (contract suite)
```

**Structure Decision**: Use existing repository layout; feature adds documentation & contract artifacts only.

## Complexity Tracking

No violations; section omitted.

## Phase 0: Research

Outputs: `research.md` consolidating decisions (seeding approach stability, metrics parity handling, background worker deferral, list serialization normalization).
Steps:
1. Catalog existing gaps (initial findings: null vs empty list, absent worker metrics updates, seed determinism).
2. Evaluate options for worker simulation (defer full worker; document recommendation).
3. Document decision matrix in research.md.

## Phase 1: Design & Contracts

Artifacts:
- `data-model.md`: Entities (GapItem, RemediationTask, ParityReport).
- `contracts/`: JSON Schema for gap inventory & closure report; minimal REST endpoint contract description (if exposed via API or CLI tooling).
- `quickstart.md`: How to run parity audit, verify tests, interpret report.
- Update agent context via script.

## Post-Design Constitution Re-Check

Expected PASS: All additions are documentation + tests; no new security surface; observability improved via documented metrics parity.

## Phase 2 (Future /tasks.md)

Will convert gap items into task entries with acceptance criteria; outside scope of `/speckit.plan`.
