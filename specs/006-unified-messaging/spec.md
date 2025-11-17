# Feature Specification: Unified Messaging

**Feature Branch**: `006-unified-messaging`  
**Created**: 2025-11-06  
**Status**: Draft  
**Input**: User description: "impl-unified-messaging With the API endpoints we want to send/receive mesages from the different types of providers.  We will not be integrating with any actual providers so we need to mock them.  We will invent some \"provider\" services for the different message types.  These mocks must provide a mixture of happy path and unhappy paths (rate limiting, server errors, etc.) so our server can correctly adapt (governors, circuit breakers etc.). We will use the inboud_events queue and some threads to consume them for different message activities (customers sending messages to contacts, messages sent by contacts originating from providers, etc). A stretch goal on this feature would be to support conversation management for automatic grouping of conversations based on to/from addresses. Test data would come from fake to support this. Data persistence if not already present would required."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Send Outbound Messages via Unified API (Priority: P1)

As a customer application, I send an outbound message (SMS, MMS, or Email) through a single unified API and the platform hands it to the correct mock provider and returns an accepted response, applying rate limiting and idempotency safeguards.

**Why this priority**: Core value proposition—enables customers to actually send messages; all other functionality builds on outbound flow.

**Independent Test**: POST a valid SMS/MMS/Email payload; receive 202 Accepted and an event recorded in the inbound events queue; duplicate POST with same Idempotency-Key yields identical accepted response without duplicate queue entry.

**Acceptance Scenarios**:
1. **Given** a valid SMS request, **When** I POST it, **Then** I receive 202 and the event is queued for provider dispatch.
2. **Given** a valid MMS request with attachments, **When** I POST it, **Then** 202 is returned and attachments are recorded in the event metadata.
3. **Given** a duplicate Email request with the same Idempotency-Key, **When** I POST again, **Then** I receive a cached 202 without a second queue entry.
4. **Given** I exceed sender rate limits, **When** I POST again, **Then** I receive 429 with retry guidance.
5. **Given** the mock provider returns a transient 500 pattern, **When** the dispatcher processes the event, **Then** the circuit breaker logic increments failure counts and may open after threshold.

---

### User Story 2 - Process Inbound Provider-Originated Messages (Priority: P1)

As the platform, I accept simulated inbound messages (webhook-like events produced by provider mocks) so that conversations can reflect replies from contacts.

**Why this priority**: Required to create two-way conversation context; without inbound messages, conversation grouping cannot be demonstrated.

**Independent Test**: Simulate (or POST) an inbound provider message event; verify it is normalized, persisted (or stored in memory if persistence already exists), and appears in a conversation listing.

**Acceptance Scenarios**:
1. **Given** a simulated inbound SMS event, **When** it's injected, **Then** it is normalized to internal schema and queued for processing.
2. **Given** an inbound Email with HTML body, **When** processed, **Then** HTML is preserved and stored.
3. **Given** malformed inbound payload missing required fields, **When** processed, **Then** it is rejected with a validation error logged (not retried endlessly).

---

### User Story 3 - Provider Mock Fault & Rate Behavior (Priority: P2)

As the platform, I encounter provider mock responses including success, rate limiting (429), and server errors (5xx) so that my circuit breaker and retry/backoff logic paths are exercised.

**Why this priority**: Ensures resilience mechanisms are validated; protects downstream systems when real providers are substituted later.

**Independent Test**: Configure mock provider failure profile (e.g., 30% 500s, 10% 429s) and send a batch of outbound messages; observe breaker opening after threshold and honoring cool-down.

**Acceptance Scenarios**:
1. **Given** a configured 429 rate limit response pattern, **When** dispatcher hits the pattern, **Then** messages are deferred/retried or surfaced appropriately.
2. **Given** consecutive 500s exceeding threshold, **When** threshold reached, **Then** breaker transitions Open and subsequent dispatch attempts short-circuit with 503 mapping until half-open window.

---

### User Story 4 - Conversation Grouping (Stretch) (Priority: P3)

As a user of the API, I can list conversations automatically grouped by unique (normalized) participant pairs regardless of direction, and list messages within each conversation.

**Why this priority**: Adds analytical and organizational value but not required for initial send/receive flows.

**Independent Test**: Send an outbound SMS and inject an inbound reply; list conversations and verify a single conversation with both messages ordered by timestamp.

**Acceptance Scenarios**:
1. **Given** at least one outbound and inbound message between the same from/to, **When** I list conversations, **Then** I see exactly one conversation containing both directions.
2. **Given** multiple message types (SMS + Email) with distinct address pairs, **When** listing conversations, **Then** they are separated by channel + address pair (assumption documented).

### Edge Cases

- Empty attachments for MMS should be rejected (already enforced in prior feature—reaffirmed here in processing pipeline). 
- Provider mock returns alternating 500/200 causing half-open transitions repeatedly—system should avoid flapping by enforcing minimum half-open sample size.
- Extremely bursty outbound submissions hitting per-sender and per-IP limits simultaneously— system returns 429 consistently and does not enqueue.
- Inbound event arrives for an unknown formatting variant (e.g., unexpected type value)— event is dropped with a single logged error (no retry loop).
- Clock skew in timestamps between outbound request and inbound reply—conversation ordering uses parsed timestamp, defaulting to ingestion time if parse fails.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept outbound unified message requests (SMS, MMS, Email) and enqueue them for provider dispatch.
- **FR-002**: System MUST apply idempotency for outbound requests using Idempotency-Key to prevent duplicate enqueue.
- **FR-003**: System MUST simulate provider dispatch via mock services producing success, 429, and 5xx responses with configurable probabilities.
- **FR-004**: System MUST implement circuit breaker transitions (Closed → Open → Half-Open → Closed) based on consecutive failures from mock providers.
- **FR-005**: System MUST process (consume) queued outbound events in background workers/threads and record dispatch outcomes.
- **FR-006**: System MUST ingest simulated inbound provider-originated messages (generated or POSTed) and normalize them to the internal event/message model.
- **FR-007**: System MUST correlate inbound and outbound messages into conversations grouped by normalized (from, to, channel) participant tuple (stretch scope—may be deferred if time constrained).
- **FR-008**: System MUST expose listing endpoints for conversations and per-conversation messages (stretch: if persistence is used, must page results; if in-memory, paging may be approximate).
- **FR-009**: System MUST enforce per-sender and per-IP rate limits on outbound submissions (reuse existing limiter) and reflect limit hits in metrics.
- **FR-010**: System MUST record metrics for: dispatch attempts, successes, failures (by type), breaker state transitions, rate-limited submissions.
- **FR-011**: System MUST provide deterministic test mode seeding so mock provider outcome distribution can be reproduced.
- **FR-012**: System MUST reject invalid inbound events with a single logged validation error and MUST NOT enqueue them.
- **FR-013**: System MUST support configuration (file/env) for mock failure/limit probabilities (e.g., success %, rate limit %, error %).
- **FR-014**: System MUST preserve message body fidelity (including HTML in email) end-to-end.
- **FR-015**: System MUST order conversation message listings by message timestamp, falling back to processing time on parse failure.

### Key Entities

- **Message**: A unit of communication (sms|mms|email); attributes: id (internal), direction (outbound|inbound), from, to, channel, body, attachments (0..n), timestamp (source), processed_at, status (pending|dispatched|failed|received).
- **Conversation**: Logical grouping keyed by (channel, normalized_from, normalized_to) independent of direction; maintains derived stats (message_count, last_activity_at).
- **DispatchAttempt**: Outcome record for one provider dispatch try: message_id, attempt_number, outcome (success|rate_limited|error), latency_ms, error_code (optional).
- **ProviderMockConfig**: Configuration for mock provider probabilities (success%, rate_limit%, error%).
- **CircuitBreakerState**: Tracks failure counts and state transitions (closed, open, half_open, last_transition_at).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 95% of valid outbound messages receive 202 response under 200 ms end-to-end acknowledgment (enqueue) in test conditions.
- **SC-002**: Circuit breaker opens within 1 second after exceeding configured consecutive failure threshold and recovers (half-open then closed) after cool-down with at least one successful trial.
- **SC-003**: Simulated failure distributions (e.g., 30% error, 10% rate limit) produce observed metrics within ±5 percentage points over 500 dispatch attempts (deterministic seed mode disabled).
- **SC-004**: Idempotent duplicate submissions do not increase queued outbound event count (0 additional entries across 50 duplicate attempts).
- **SC-005**: Conversation grouping (stretch) correctly aggregates ≥ 95% of bidirectional message pairs in test dataset without manual intervention.
- **SC-006**: Inbound malformed events are rejected with 0 successful insertions and 100% single-pass validation logging (no retries observed in logs) in controlled test batch.
- **SC-007**: Metrics endpoint reflects breaker state changes within 500 ms of transition under load test.

## Assumptions

- Provider mocks are internal components, not separate processes (keeps scope contained). 
- Persistence layer exists from prior features or can fallback to in-memory storage without changing API surface.
- Conversation grouping excludes cross-channel merging (SMS vs Email kept separate even if addresses conceptually match).
- Failure probability configuration applied globally per channel (not per-sender for initial scope).
- Deterministic seed mode toggled via environment variable.

## Out of Scope

- Real third-party provider integrations.
- Message content filtering or compliance scanning.
- Attachments storage beyond URL reference validation.
- Multi-tenant isolation concerns.

## Dependencies

- Requires existing rate limiter, circuit breaker, idempotency infrastructure (already delivered in earlier features).
- Reuses inbound_events queue; may require capacity adjustment for load testing.

## Risks

- Flapping breaker if half-open sampling logic too small.
- Memory growth if dispatch retry policy not bounded.
- Non-deterministic tests if failure probabilities not seed-controlled.

## Open Questions

None (all reasonable defaults selected; no critical clarifications required within scope constraints).
