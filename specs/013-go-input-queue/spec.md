# Feature Specification: Go Input Events Queue for Messaging Server (Go parity with Rust)

**Feature Branch**: `013-go-input-queue`  
**Created**: 2025-11-14  
**Status**: Draft  
**Input**: User description: "go-input-queue The Go server needs to be equivalent to the the Rust server. The Rust server's HTTP handlers queue valid requests to an input events queue. Then the worker finds any pending work and performs the SQL persistence and HTTP requests \"offline\" from the original request. You are to properly create the new feature to handle this work."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Enqueue valid outbound requests (Priority: P1)

As an API client and operator, I need the Go HTTP handlers to validate outbound SMS/Email requests and enqueue normalized input-events instead of performing persistence/provider operations inline, so requests return fast and the system remains resilient under load.

**Why this priority**: Decoupling HTTP from heavy work is essential to meet latency and reliability goals while matching the Rust reference architecture.

**Independent Test**: Run the Go server with a test input-events queue, submit valid and invalid SMS/Email requests, and assert that valid requests get HTTP 202 and exactly one event on the queue; invalid requests return 4xx and do not enqueue any event.

**Acceptance Scenarios**:

1. **Given** a valid SMS request and a healthy queue, **When** POSTing to `/api/messages/sms`, **Then** the server returns HTTP 202 and one SMS event is enqueued.
2. **Given** an invalid Email request (e.g., empty body), **When** POSTing to `/api/messages/email`, **Then** the server returns HTTP 400 and no event is enqueued.

---

### User Story 2 - Worker processes queued events (Priority: P2)

As an operator, I need a worker that consumes queued events and applies durable persistence (conversation upsert, body de-duplication, idempotent message insertion, conversation counters/timestamps) and initiates downstream provider actions, so data integrity and delivery are ensured without blocking HTTP.

**Why this priority**: Enqueueing alone is insufficient; a robust worker must process events reliably and idempotently to match Rust behavior.

**Independent Test**: Seed known events on the queue and run the worker connected to a test database; assert final DB state matches expectations and replaying the same events produces no duplicates.

**Acceptance Scenarios**:

1. **Given** a valid outbound SMS event on the queue and DB connectivity, **When** the worker processes it, **Then** a single outbound message and corresponding conversation exist in the database and the event is acknowledged.
2. **Given** a duplicate event (same idempotency key), **When** processed, **Then** no additional message is created and conversation counters remain correct.

---

### User Story 3 - Configuration and visibility (Priority: P3)

As an operator, I need configuration to select the queue backend (in-memory for dev, persistent for prod) and basic metrics/logging to observe enqueue/processing rates, failures, and backlog, so I can run and troubleshoot the system confidently.

**Why this priority**: Operability and environment parity are needed to deploy safely.

**Independent Test**: Run with different queue modes, simulate failures (DB down, provider errors), and verify metrics/logs reflect health/backlog and errors include actionable context.

**Acceptance Scenarios**:

1. **Given** the system is healthy, **When** observing metrics/logs, **Then** enqueue/processing rates are visible and backlog is near zero.
2. **Given** the DB is unavailable, **When** the worker processes events, **Then** failures are logged with sufficient context and events are retried according to policy without being lost silently.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

- Queue unavailable at enqueue time (HTTP should return 503/500 with clear error and no partial event written; retries may be client-controlled).
- Events referencing non-existent customers/conversations (worker creates conversations as needed; customer assignment follows configured default). 
- Worker crash during processing (at-least-once with idempotency ensures no duplicates on restart).
- Malformed/legacy events on the queue (worker rejects with clear error and moves to dead-letter after retry policy). 
- Server/worker version skew during deploy (event schema forwards/backwards compatibility plan).

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST validate outbound SMS/Email HTTP requests and enqueue a normalized input-event for each valid request; invalid requests MUST be rejected with 4xx and not enqueued.
- **FR-002**: System MUST provide a worker that consumes events and performs durable persistence (conversation upsert, body de-duplication, idempotent message insertion, conversation counters/timestamps) and initiates downstream provider actions where applicable.
- **FR-003**: System MUST ensure idempotent processing such that replaying the same event does not create duplicate messages or corrupt counts.
- **FR-004**: System MUST allow server and worker to be deployed and scaled independently while preserving consistent behavior.
- **FR-005**: System MUST expose configuration for queue mode (e.g., in-memory vs persistent) and worker concurrency/retry policy.
- **FR-006**: System MUST provide logs and/or metrics sufficient to detect enqueue failures, processing errors, and backlog growth.
- **FR-007**: System MUST document the event schema and idempotency key derivation to ensure compatibility and debuggability across components.
- **FR-008**: System MUST support at-least-once delivery semantics with idempotent processing to guarantee correctness under retries.
- **FR-009**: System MUST define failure handling for unprocessable events as follows: retry with capped exponential backoff for up to 72 hours (maximum 10 attempts). After retries are exhausted or the window elapses, move the event to a dead-letter queue retained for 30 days (with sufficient metadata for investigation and replay).

### Key Entities *(include if feature involves data)*

- **Outbound Message Event**: Standardized representation for outbound SMS/Email, including channel, participants, body, attachments, timestamps, and idempotency key.
- **Input Events Queue**: Logical queue abstraction that stores events for worker consumption with ordering, retry, and failure handling semantics.
- **Worker Execution**: Consumer component responsible for event processing, persistence, downstream actions, and emitting metrics/logs.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: 95% of valid outbound HTTP requests receive a response within 200ms under normal load, independent of downstream DB/provider latency.
- **SC-002**: In test environments, the worker processes at least 10,000 outbound events/hour without message loss or duplication.
- **SC-003**: For a representative test batch, the Go queue implementation produces the same final persisted state as the Rust reference for the same inputs.
- **SC-004**: Operators can determine queue/worker health and backlog state within 5 minutes via metrics/logs, without inspecting DB internals.
