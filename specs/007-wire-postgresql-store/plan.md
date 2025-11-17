# Implementation Plan: Wire PostgreSQL Store

**Branch**: `007-wire-postgresql-store` | **Date**: 2025-11-06 | **Spec**: ../spec.md
**Input**: Feature specification from `/specs/007-wire-postgresql-store/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See project documentation for the execution workflow.

## Summary

Durably persist inbound events in PostgreSQL and introduce a background worker that claims and processes those events into normalized messages, migrating conversations/messages reads to DB while preserving existing API contracts. Technical approach: SQLx-backed tables (inbound_events, messages, conversations), idempotency via unique constraints, worker claim loop with SKIP LOCKED, retries with exponential backoff, and metrics/tracing to observe throughput and failures.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust (stable; repo targets 1.75+)  
**Primary Dependencies**: Axum, Tokio, SQLx, Serde, Tracing  
**Storage**: PostgreSQL (docker-compose; SQLx migrations)  
**Testing**: cargo test; HTTP harness at bin/test.sh with jq schema  
**Target Platform**: Linux/macOS server  
**Project Type**: Backend service  
**Performance Goals**: SC-001: 95% events processed <5s single instance  
**Constraints**: Idempotency for inbound; durable write-before-ack  
**Scale/Scope**: Dev-scale; supports parallel workers via SKIP LOCKED

## Constitution Check

GATE: PASS (pre-design)

- Security-First: Input validation persists; provider payloads stored as JSONB with size caps and redaction; secrets not logged. PASS
- Test-First: Add integration tests for worker claim/processing and idempotent inserts; maintain coverage. PASS
- Observability: Metrics for queued/claimed/processed/errored; structured logs and traces for event ids. PASS
- Versioning/Change Control: Internal-only change; no breaking API; migrations documented. PASS
- Simplicity: Use direct SQLx (no heavy ORM), minimal worker loop complexity. PASS

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
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: Single backend project under crates/; new DB-backed stores and worker modules will live in crates/server/src/{store_db,worker}/ with migrations under migrations_sqlx/ managed by db-migrate.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
