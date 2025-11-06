# Implementation Plan: Unified Messaging

**Branch**: `006-unified-messaging` | **Date**: 2025-11-06 | **Spec**: specs/006-unified-messaging/spec.md
**Input**: Feature specification from `/specs/006-unified-messaging/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See project documentation for the execution workflow.

## Summary

Implement a unified messaging capability using existing API endpoints to send and receive messages across SMS/MMS/Email with internal mock providers. Mocks expose configurable mixtures of success, rate limiting (429), and server errors (5xx) to exercise resilience (rate limiters, circuit breakers). Outbound requests are enqueued and processed by background workers; inbound provider-originated events are normalized and ingested. Stretch scope: conversation grouping by normalized participant pairs.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.75+ (repo currently builds on stable)  
**Primary Dependencies**: Axum (HTTP), Tokio (async runtime), Serde, Tracing; internal modules for rate limiting, circuit breaker, idempotency, inbound queue  
**Storage**: PostgreSQL via SQLx planned; for this feature, in-memory acceptable if persistence not yet wired for messages; conversation stretch may use in-memory grouping  
**Testing**: cargo test; HTTP contract tests via `bin/test.sh` and JSON cases; additional scenario tests for provider mocks  
**Target Platform**: Linux/macOS server (local dev), containerized deployment-ready  
**Project Type**: Single backend service  
**Performance Goals**: 95% of enqueue acks < 200 ms; breaker transition within 1 s after threshold  
**Constraints**: Respect existing rate limits and body size limits; redaction for sensitive headers; deterministic seeding optional  
**Scale/Scope**: Tens of RPS for local validation; synthetic load for breaker/rate scenarios

## Constitution Check

Security-First: No real provider secrets; mocks used only; ensure no sensitive data logged; continue redaction in logs.  
Test-First and Quality Gates: Expand tests to include failure distributions; maintain contract tests; run clippy and unit tests.  
Observability: Maintain structured request logs and metrics; add counters for dispatch outcomes and breaker transitions.  
Versioning and Change Control: Non-breaking additions; update OpenAPI under feature contracts.  
Simplicity: Keep mocks in-process with clear seams; avoid over-engineering retry policy.

Gate status: PASS (no violations).

Update 2025-11-06: Contracts (openapi.yaml) and Quickstart added; inbound injection and mock runtime config endpoints defined. Observability, redaction, and test-first notes remain valid. No new risks introduced.

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
specs/006-unified-messaging/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/

crates/server/
├── src/
│   ├── api/
│   ├── middleware/
│   ├── queue/
│   ├── config.rs
│   └── metrics.rs
└── tests/

tests/http/
└── tests.json
```

**Structure Decision**: Single backend service; feature docs under specs/006; provider mocks and workers implemented inside `crates/server` with minimal surface area.

## Complexity Tracking

No violations.
