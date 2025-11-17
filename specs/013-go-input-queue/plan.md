# Implementation Plan: Go Input Events Queue for Messaging Server (Go parity with Rust)

**Branch**: `013-go-input-queue` | **Date**: 2025-11-14 | **Spec**: specs/013-go-input-queue/spec.md
**Input**: Feature specification from `/specs/013-go-input-queue/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See project documentation for the execution workflow.

## Summary

Make the Go server equivalent to the Rust server by decoupling HTTP request handling from persistence and delivery using an input-events queue and a background worker. HTTP handlers validate requests, enqueue events, and return 202. The worker consumes events and performs durable persistence (conversation upsert, message body de-duplication, idempotent insert) and triggers downstream actions. Reliability is ensured via at-least-once processing with idempotent semantics; failed events follow a retry policy and DLQ retention as specified.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.

**Language/Version**: Go 1.22+ (Rust remains the reference implementation)  
**Primary Dependencies**: chi (HTTP), pgx/sqlc (DB), zap (logging), Prometheus client (metrics); new internal queue abstraction  
**Storage**: PostgreSQL (primary). In-memory acceptable for queue backend initially (dev/test).  
**Testing**: `go test` for unit/integration, repository tests with real DB; worker tests with in-memory queue + test DB  
**Target Platform**: Linux/macOS server processes (server + worker can run in one or separate processes)
**Project Type**: Backend service (HTTP API + background worker)  
**Performance Goals**: p95 handler latency ≤200ms when enqueueing; sustain ≥10k events/hour without loss/duplication  
**Constraints**: Maintain security headers and rate limiting; strict idempotency; structured logging and metrics  
**Scale/Scope**: Introduce a queue interface with an in-memory implementation now; leave room for a persistent backend later; worker concurrency is configurable but modest by default

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Security-First and Secrets Hygiene: PASS (no new secrets; environment variables for config; redact sensitive data in logs)
- Test-First and Quality Gates: PASS (add unit + integration tests for handlers, queue, and worker; keep CI green)
- Observability and Auditability: PASS (structured logs; metrics for enqueue, dequeued, processed, failed, retries, DLQ size)
- Backwards Compatibility/Versioning: PASS (no API breaking changes; same endpoints and request schemas)
- Simplicity and Single Responsibility: PASS (clean queue interface; worker handles persistence; handlers only enqueue)

## Project Structure

### Documentation (this feature)

```text
specs/013-go-input-queue/
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

```text
go/
├── api/                      # existing HTTP handlers (unchanged endpoints; produce events)
├── internal/
│  ├── queue/                 # NEW: input-events queue abstraction + in-memory impl
│  │  ├── queue.go            # interface + event type(s)
│  │  └── memory/
│  │     └── memory.go        # in-memory implementation
│  ├── worker/                # NEW: worker that consumes events and calls repositories
│  │  └── worker.go
│  ├── db/
│  │  ├── repository/         # existing repositories (messages, conversations)
│  │  └── store/              # reintroduce Create*Message used by worker
│  ├── metrics/               # existing metrics integration + new queue/worker metrics
│  └── middleware/            # existing security/auth/rate-limiting
└── cmd/
  ├── server/                # wire queue producer in handlers; persistence moves to worker
  └── worker/                # optional separate binary to run worker (or embedded goroutine)
```

**Structure Decision**: Single backend with new `internal/queue` and `internal/worker` packages. The server can run the worker in-process or as a separate binary under `cmd/worker`.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|---------------------------------------|
| None | — | — |

---

## Phase 0: Outline & Research

Unknowns resolved and rationale:
- DLQ/Retention Policy: Retry up to 72 hours with capped exponential backoff (max 10 attempts). Then move to DLQ retained 30 days. Chosen Option B from spec for safety and operability.
- Idempotency Key: Deterministically derived from channel + participants + normalized timestamp + body hash to match current DB semantics and support at-least-once processing.
- Queue Backend: Start with in-memory channel for dev/tests to minimize scope; design queue interface to allow future persistent backends.

Deliverable: `specs/013-go-input-queue/research.md` documents decisions, alternatives, and edge cases.

## Phase 1: Design & Contracts

- Data Model: Define `OutboundMessageEvent` with fields needed for persistence and downstream routing; include schema versioning.
- Contracts: Provide OpenAPI for POST `/api/messages/sms` and `/api/messages/email` confirming 202 async semantics. Provide JSON Schema for the event.
- Quickstart: Document how to run server + worker with in-memory queue; environment variables; observability.
- Agent Context: Update Copilot agent context with new components via the provided script.

Re-check Constitution: PASS. No new violations.

Outputs:
- specs/013-go-input-queue/research.md
- specs/013-go-input-queue/data-model.md
- specs/013-go-input-queue/contracts/openapi.yaml
- specs/013-go-input-queue/contracts/event.schema.json
- specs/013-go-input-queue/quickstart.md
