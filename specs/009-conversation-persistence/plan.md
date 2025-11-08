# Implementation Plan: Conversation Persistence & Unification

**Branch**: `009-conversation-persistence` | **Date**: 2025-11-07 | **Spec**: specs/009-conversation-persistence/spec.md
**Input**: Feature specification from `/specs/009-conversation-persistence/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See project documentation for the execution workflow.

## Summary

Implement a unified, durable conversation model ensuring every inbound/outbound message is atomically associated with a single persistent conversation keyed by (channel + normalized ordered participants). Provide deterministic listing/pagination, accurate counts & activity timestamps, safe UTF-8 snippets, idempotent backfill, and metrics/observability. Technical approach: add conversation table + FK on messages; implement normalization module; transactional upsert (INSERT .. ON CONFLICT) updating counts; migration to backfill legacy messages; API DTO adjustments; metrics counters and structured logging.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.75 (stable toolchain)  
**Primary Dependencies**: Axum (HTTP), Tokio (async), SQLx (PostgreSQL), Serde (serialization), Tracing (observability)  
**Storage**: PostgreSQL (durable conversations/messages); in-memory only for fallback when DB unavailable  
**Testing**: cargo test (unit/integration), potential cargo nextest (future), load tests via Tokio tasks  
**Target Platform**: Linux container / macOS local dev; server-side only  
**Project Type**: Multi-crate backend service (`crates/core`, `crates/server`, `crates/db-migrate`, `crates/admin`)  
**Performance Goals**: Conversation upsert P95 ≤10ms local, ≤25ms at 100 RPS; zero duplicate conversation creation under race  
**Constraints**: Atomic message+conversation transaction; safe UTF-8 snippet truncation; unique index enforcement; maintain backward-compatible API fields  
**Scale/Scope**: Initially thousands of conversations; designed for growth to millions (unique index & timestamp index)  

**Unknowns / NEEDS CLARIFICATION**: NONE (resolved in spec: plus-addressing normalization, channels limited to Email & SMS/MMS)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Principle Alignment:
- Security-First: No secrets introduced; input normalization is deterministic & sanitized; no PII beyond addresses which are normalized.
- Test-First: Plan includes unit tests (normalization, transactional upsert), integration tests (outbound+inbound pair), load test concurrency.
- Observability: Structured logs (message_id, conversation_key), metrics counters (created/reused/failures).
- Versioning & Compatibility: Adds columns/tables with additive migrations; API extended without breaking existing fields.
- Simplicity: Single normalization module; application-level transaction preferred over triggers to avoid complexity.

Gate Verdict (Pre-Design): PASS (no violations). Complexity Tracking section not required.

Post-Design Re-evaluation:
- Added artifacts: research.md, data-model.md, contracts/openapi.yaml, quickstart.md.
- Security: No secrets added; normalization logic purely functional.
- Tests (planned): Unit (normalization, upsert), integration (pair flow, pagination), load (100 concurrent inserts).
- Observability: Metrics counters and structured logs remain in scope; no new gates introduced.
- Compatibility: Additive schema; API extended without breaking fields.

Post-Design Gate Verdict: PASS.

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
crates/
  core/
    src/
      conversations/            # New normalization + key derivation module
      messaging/                # Existing message logic (will be updated)
  server/
    src/
      api/
        conversations.rs        # Conversation list & messages endpoints adjustments
      config/                   # Extend for snippet length configuration
  db-migrate/
    migrations_sqlx/            # New migrations for conversations table & message FK
    src/
      backfill_conversations.rs # Backfill utility logic (Phase 1)
tests/
  integration/
    conversation_flow.rs        # Outbound + inbound pair test
    concurrency_upsert.rs       # 100 simultaneous inserts test
  unit/
    normalization_email.rs
    normalization_phone.rs
    upsert_logic.rs
specs/009-conversation-persistence/
  research.md
  data-model.md
  contracts/
    openapi.yaml
  quickstart.md
```

**Structure Decision**: Extend existing multi-crate backend; place normalization & upsert logic in `core`, API adjustments in `server`, migrations/backfill in `db-migrate`, tests in dedicated unit/integration folders; additive docs within feature specs directory.

## Complexity Tracking

Not required (no constitution gate violations).
