# Feature Specification: Wire PostgreSQL Store

**Feature Branch**: `007-wire-postgresql-store`  
**Created**: 2025-11-06  
**Status**: Draft  
**Input**: User description: "wire-postgresql-store There should now be a background process that looks for inbound_events to process. These inbound_events would be stored in PostgreSQL, and now it is the time to implement the logic for these events."

## User Scenarios & Testing (mandatory)

### User Story 1 - Persist inbound events (Priority: P1)

As a developer/operator, I want inbound events (produced by provider webhooks or internal normalization) to be stored durably in PostgreSQL so that they survive restarts and can be processed exactly-once by a background worker.

**Why this priority**: Without durable storage, events are lost on restart and processing cannot be audited or retried.

**Independent Test**: Post an inbound webhook; verify a new row in the inbound_events table with status="received" and the original payload. Restart the service and confirm the row remains.

**Acceptance Scenarios**:

1. Given a valid inbound webhook payload, When it is accepted by the API, Then a new inbound_event record is inserted with status="received", normalized keys (channel/from/to), and a created_at timestamp.
2. Given a duplicate inbound webhook (same provider message id + channel), When posted again, Then the API returns 202 and no duplicate row is created (idempotent insert).

---

### User Story 2 - Background worker processes events (Priority: P1)

As a system, I want a background process to continuously claim and process unprocessed inbound_events, updating their status and producing normalized Message records so downstream conversation APIs work.

**Why this priority**: Enables end-to-end functionality and unlocks conversation listing and metrics based on durable data.

**Independent Test**: Insert a synthetic inbound_event row with status="received"; run worker; verify status transitions to "processed" with processed_at, and a Message row is inserted.

**Acceptance Scenarios**:

1. Given inbound_events with status="received", When the worker runs, Then it atomically claims a batch (status="processing", processor_id set) and processes them.
2. Given successful processing, When the worker completes, Then the inbound_event status becomes "processed" and a Message row is persisted referencing the event id.
3. Given a processing error, When it occurs, Then the inbound_event status becomes "error", error fields are captured, and the worker resumes with backoff; after threshold K errors the item is parked as "dead_letter".

---

### User Story 3 - Conversations read from DB (Priority: P2)

As an API consumer, I want the conversations and messages listing endpoints to read from PostgreSQL rather than memory so that history is consistent and queryable.

**Why this priority**: Aligns read paths with durable writes; enables pagination and future indexing.

**Independent Test**: After processing events, call GET /api/conversations and /api/conversations/{id}/messages and verify results reflect DB contents and pagination metadata.

**Acceptance Scenarios**:

1. Given persisted Message rows, When listing conversations, Then results are aggregated by normalized keys and ordered by last_activity_at desc.
2. Given a conversation id, When listing messages, Then results are ordered by timestamp asc and limited by page/pageSize.

---

### Edge Cases

- Event duplication (same provider id) across retries must not produce duplicates (unique index on channel+provider_message_id).
- Out-of-order timestamps should not break conversation ordering (last_activity_at uses max of message timestamps).
- Large payloads should be truncated or stored in a JSONB column with size guardrails.
- Processing crashes mid-batch must release claims (heartbeat/timeout) to avoid stuck events.

## Requirements (mandatory)

### Functional Requirements

- FR-001: API MUST persist inbound events to PostgreSQL with fields: id (ULID/UUID), channel, from, to, provider_message_id, payload JSONB, status (received|processing|processed|error|dead_letter), created_at, updated_at, processed_at?, error_code?, error_message?, processor_id?.
- FR-002: System MUST enforce idempotency for inbound events via unique constraint on (channel, provider_message_id) and safe upsert behavior.
- FR-003: A background worker MUST periodically claim and process events atomically using SELECT ... FOR UPDATE SKIP LOCKED (or equivalent) and update statuses.
- FR-004: Processing MUST produce normalized Message rows linked to inbound_event_id, suitable for conversation grouping and listing queries.
- FR-005: On processing errors, system MUST record error details and retry with exponential backoff up to a configured limit; exceeded items MUST be marked dead_letter.
- FR-006: Conversation and message listing endpoints MUST read from PostgreSQL with pagination parameters page and pageSize; responses MUST include total.
- FR-007: Configuration MUST allow tuning batch size, claim timeout, backoff strategy, and max retries via env/file.
- FR-008: Metrics MUST expose counters/gauges for queued, claimed, processed, errored, dead_letter, and processing latency percentiles.
- FR-009: Tracing MUST include correlation ids and event ids across API receipt and worker processing spans.
- FR-010: Migrations MUST create necessary tables, indexes, and constraints; offline build MUST succeed with SQLX_OFFLINE=true.

### Key Entities

- InboundEvent: id, channel, from, to, provider_message_id, payload, status, created_at, updated_at, processed_at, error_code, error_message, processor_id, attempt_count, next_attempt_at.
- Message: id, inbound_event_id (nullable for outbound), channel, from, to, body, attachments, timestamp.
- Conversation: id, key (channel/from/to normalized, sorted), message_count, last_activity_at.

## Success Criteria (mandatory)

### Measurable Outcomes

- SC-001: 95% of inbound events transition from received to processed within 5 seconds under nominal load (single instance).
- SC-002: 0 duplicate Message rows for the same provider_message_id and channel during retries (verified by unique constraints).
- SC-003: Conversation listing reflects processed events within 2 seconds of processing completion.
- SC-004: On crash/restart, at-least-once processing ensures no stuck events for more than 60 seconds (claim timeout/heartbeat reapplies).

# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`  
**Created**: [DATE]  
**Status**: Draft  
**Input**: User description: "$ARGUMENTS"

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

### User Story 1 - [Brief Title] (Priority: P1)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently - e.g., "Can be fully tested by [specific action] and delivers [specific value]"]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 2 - [Brief Title] (Priority: P2)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 3 - [Brief Title] (Priority: P3)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right edge cases.
-->

- What happens when [boundary condition]?
- How does system handle [error scenario]?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST [specific capability, e.g., "allow users to create accounts"]
- **FR-002**: System MUST [specific capability, e.g., "validate email addresses"]  
- **FR-003**: Users MUST be able to [key interaction, e.g., "reset their password"]
- **FR-004**: System MUST [data requirement, e.g., "persist user preferences"]
- **FR-005**: System MUST [behavior, e.g., "log all security events"]

*Example of marking unclear requirements:*

- **FR-006**: System MUST authenticate users via [NEEDS CLARIFICATION: auth method not specified - email/password, SSO, OAuth?]
- **FR-007**: System MUST retain user data for [NEEDS CLARIFICATION: retention period not specified]

### Key Entities *(include if feature involves data)*

- **[Entity 1]**: [What it represents, key attributes without implementation]
- **[Entity 2]**: [What it represents, relationships to other entities]

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: [Measurable metric, e.g., "Users can complete account creation in under 2 minutes"]
- **SC-002**: [Measurable metric, e.g., "System handles 1000 concurrent users without degradation"]
- **SC-003**: [User satisfaction metric, e.g., "90% of users successfully complete primary task on first attempt"]
- **SC-004**: [Business metric, e.g., "Reduce support tickets related to [X] by 50%"]
