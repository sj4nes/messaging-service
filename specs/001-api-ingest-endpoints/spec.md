# Feature Specification: API Ingest Endpoints

**Feature Branch**: `001-api-ingest-endpoints`  
**Created**: 2025-11-05  
**Status**: Draft  
**Input**: User description: "int-axum-server: Support API calls; GET endpoints return requested Content-Type; POST endpoints enqueue inbound_events for async processing; validate request parameters and types; add per-IP and per-sender rate limits and webhook circuit breakers; integrate with existing bin/test.sh endpoints (messages sms/email, webhooks sms/email, conversations list and messages)."

## Clarifications

### Session 2025-11-06

- Q: Should POST endpoints also have per-IP and per-sender rate limits? → A: Yes — apply per-IP and per-sender rate limits to message POST endpoints.

## User Scenarios & Testing (mandatory)

### User Story 1 - Ingest messages via POST (Priority: P1)

As a client or provider, I can POST message payloads to API endpoints (SMS/MMS, Email) that are validated and accepted, and the system enqueues them for asynchronous processing, returning an immediate acknowledgement without blocking on downstream processing.

**Why this priority**: Enables core message ingestion with reliable capture and backpressure isolation; unblocks downstream processing by other components.

**Independent Test**: Run the provided `bin/test.sh` POST calls to `/api/messages/sms` and `/api/messages/email`; verify 202 Accepted (or 200 OK with body), response headers and IDs, and that a queue record is created.

**Acceptance Scenarios**:

1. Given a valid SMS payload (type=sms), when POSTing to `/api/messages/sms` with `Content-Type: application/json`, then the request is validated, an inbound event is enqueued, and the API returns 202 Accepted with an event identifier.
2. Given a valid MMS payload (type=mms with attachments), when POSTing to `/api/messages/sms`, then validation includes attachment URLs and size limits, the event is enqueued, and the API returns 202 Accepted.
3. Given a valid Email payload, when POSTing to `/api/messages/email`, then the event is enqueued and the API returns 202 Accepted.
4. Given an invalid payload (missing required fields, wrong types), when POSTing, then the API returns 400 with a machine-readable error body and no event is enqueued.

---

### User Story 2 - Ingest provider webhooks with safeguards (Priority: P1)

As a provider integration, I can POST incoming webhook payloads (SMS/Email) and the API will enforce per-IP and per-sender rate limits and circuit breakers to protect the service, while still reliably enqueuing valid events.

**Why this priority**: Prevents abuse and cascading failures; ensures reliability of inbound flows from third parties.

**Independent Test**: Use `bin/test.sh` webhook POSTs to `/api/webhooks/sms` and `/api/webhooks/email`; simulate bursts from the same IP or sender to observe rate limiting (429) and circuit breaker open state (503), verify proper headers and reset times.

**Acceptance Scenarios**:

1. Given repeated webhook calls from the same IP exceeding configured thresholds, when POSTing to `/api/webhooks/sms`, then responses become 429 Too Many Requests with reset/limit headers and bodies, and valid calls below limits are enqueued.
2. Given a sustained upstream failure pattern, when the circuit breaker opens for a webhook route, then the API returns 503 Service Unavailable with a retry hint until the breaker half-opens/resets, and no queue inserts occur during open state.
3. Given valid payloads under thresholds, when POSTing, then the event is enqueued and 202 Accepted is returned.

---

### User Story 3 - Retrieve conversations (Priority: P2)

As a client, I can GET a list of conversations and receive responses in the requested content type (default JSON), with pagination and filters to support UI and automation.

**Why this priority**: Enables read-side integrations and basic visibility without coupling to storage internals.

**Independent Test**: Call `GET /api/conversations` with `Accept: application/json` (or default), verify 200 OK with a pageable list and metadata; validate schema and headers.

**Acceptance Scenarios**:

1. Given existing conversations, when GET `/api/conversations`, then the response is 200 with a JSON array plus paging metadata (limit, offset/next token, total or has_more).
2. Given a non-default Accept header (e.g., `application/json`), when GET `/api/conversations`, then the server responds with that content type if supported; otherwise responds 406 Not Acceptable.
3. Given large result sets, when using pagination parameters, then results are stable and consistent across pages.

---

### User Story 4 - Retrieve messages for a conversation (Priority: P2)

As a client, I can GET messages for a conversation ID, receiving the requested content type (default JSON) with pagination.

**Why this priority**: Supports UI and automation for message history per conversation.

**Independent Test**: Call `GET /api/conversations/{id}/messages` and verify 200 OK with messages and paging metadata; 404 for unknown conversation.

**Acceptance Scenarios**:

1. Given a valid conversation ID with messages, when GET `/api/conversations/1/messages`, then 200 OK with a JSON array sorted by time and paging metadata.
2. Given an unknown conversation ID, when GET `/api/conversations/999/messages`, then 404 Not Found with a machine-readable error body.

### Edge Cases

- Payload exceeds maximum body size limit → 413 Payload Too Large; request aborted before enqueue.
- Missing or incorrect `Content-Type` → 415 Unsupported Media Type; no enqueue.
- Unsupported `Accept` header → 406 Not Acceptable.
- Duplicate POST with same idempotency key (if provided) → 200/202 idempotent success with same identifier; no duplicate enqueue.
- Sender or IP exceeds rate limit → 429 with limit info headers; request may or may not enqueue depending on policy (default: do not enqueue when throttled).
- Circuit breaker open for a route → 503 with retry-after guidance; no enqueue until half-open.
- Malformed attachment URLs or disallowed schemes → 400 with specific error path; no enqueue.
- Sender or IP exceeds rate limit on POST endpoints → 429 with limit info headers; request is rejected and not enqueued.

## Requirements (mandatory)

### Functional Requirements

- FR-001: The API MUST expose POST endpoints for message ingestion at `/api/messages/sms` and `/api/messages/email` accepting JSON bodies.
- FR-002: The API MUST validate request bodies (types, required fields, size constraints) and reject invalid requests with 400 and a machine-readable error schema.
- FR-003: On successful validation, the API MUST enqueue an inbound event record into the inbound_events queue for asynchronous processing and return 202 Accepted with an identifier.
- FR-004: The API MUST enforce `Content-Type: application/json` for POST requests and respond with 415 for unsupported media types.
- FR-005: The API MUST support content negotiation on GET endpoints and return the requested content type when supported (default JSON); otherwise respond 406.
- FR-006: The API MUST provide GET endpoints `GET /api/conversations` (pageable list) and `GET /api/conversations/{id}/messages` (pageable list) with stable pagination.
- FR-007: The API MUST implement per-IP and per-sender rate limiting for webhook endpoints (`/api/webhooks/sms`, `/api/webhooks/email`) with standard 429 responses and limit/reset headers.
- FR-008: The API MUST implement circuit breakers for webhook endpoints to shed load during sustained error conditions, returning 503 while open and auto-recovering per policy.
- FR-009: The API SHOULD support idempotency keys for POST requests; when provided, duplicate requests MUST not create duplicate queue entries and SHOULD return the original acknowledgement.
- FR-010: The API MUST cap request body size and attachment list/URL sizes to configured limits; exceeding limits results in 413.
- FR-011: The API MUST reject payloads containing disallowed URL schemes or invalid URLs in attachments with 400.
- FR-012: The API MUST return consistent error responses with `code`, `message`, and optional `details` fields.
- FR-013: The system MUST log acceptance and rejection events for auditability without logging sensitive content; include correlation IDs.
- FR-014: The system MUST expose minimal operational metrics (e.g., counts, rates, throttles, breaker state) for monitoring.
- FR-015: The API MUST align with the domain events envelope semantics for identifiers and timestamps; enqueued records MUST be compatible with the envelope used by downstream processors.
- FR-016: The API MUST implement per-IP and per-sender rate limiting for message ingestion POST endpoints (`/api/messages/sms`, `/api/messages/email`) with standard 429 responses and limit/reset headers; limits are configurable and SHOULD be consistent with webhook policy unless otherwise specified.

### Key Entities (data-facing, conceptual)

- InboundEvent: A validated, accepted unit placed onto the inbound_events queue; includes event_id, occurred_at, aggregate identifiers, type, source (api/webhook), and optional idempotency_key.
- Sender: The logical source address (e.g., phone number or email) used for per-sender limits.
- RateLimitCounter: Rolling counters and limits applied per-IP and per-sender for webhook routes.
- CircuitBreaker: State machine (closed → open → half-open) per webhook route and/or provider.
- Conversation: Logical thread of messages; list view for GET with paging metadata.
- MessageSummary: Message items within a conversation list; excludes sensitive bodies when not necessary.

## Success Criteria (mandatory)

### Measurable Outcomes

- SC-001: 95% of valid POST requests return an acknowledgement in ≤ 200 ms under nominal load (since processing is asynchronous).
- SC-002: 0 invalid records are enqueued (schema/type validation blocks malformed payloads).
- SC-003: GET endpoints return a valid JSON response with correct paging metadata in 99% of calls under nominal load.
- SC-004: Rate limiting correctly throttles abusive sources: at least 99% of bursts over the configured limit receive 429 without enqueue.
- SC-005: When breaker conditions are met, 100% of requests during open state receive 503 and 0 are enqueued; breaker auto-recovers per policy (verified in integration tests).

---

Notes: This specification is technology-agnostic. Security and validation idioms (content type enforcement, size limits, rate limits, breaker behavior, and idempotency) follow industry best practices until further guidance is provided.
