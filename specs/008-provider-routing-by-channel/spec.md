# Feature Specification: Provider Routing By Channel

**Feature Branch**: `008-provider-routing-by-channel`  
**Created**: 2025-11-07  
**Status**: Draft  
**Input**: User description: "Implement explicit provider routing for outbound messages by channel (SMS, MMS, Email); per-provider breaker isolation and metrics; record provider identity; keep endpoints unchanged."

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

### User Story 1 - Outbound Messages Route to Correct Provider (Priority: P1)

As a client application submitting outbound messages (SMS, MMS, Email), I receive an accepted response and each message is internally routed to the provider component dedicated to its channel, enabling reliable dispatch behavior and future extensibility.

**Why this priority**: Core functional gap; without routing there is no true multi-provider capability or isolation.

**Independent Test**: POST sample SMS, MMS, Email payloads; verify 202 responses and logs showing provider="sms-mms" for SMS/MMS and provider="email" for Email; metrics counters per provider increment.

**Acceptance Scenarios**:
1. **Given** a valid SMS request, **When** I POST to `/api/messages/sms`, **Then** the system enqueues it and logs a dispatch attempt with provider="sms-mms".
2. **Given** a valid Email request, **When** I POST to `/api/messages/email`, **Then** the system enqueues it and logs a dispatch attempt with provider="email".
3. **Given** a valid MMS request, **When** I POST to `/api/messages/sms` (type=mms), **Then** it routes via the sms-mms provider with distinct metrics label.

---
### User Story 2 - Provider Health Isolation & Circuit Breaking (Priority: P2)

As an operator monitoring reliability, I can observe failures in one provider (e.g., repeated errors for SMS/MMS) causing only its breaker to open while other providers continue dispatching, ensuring fault isolation.

**Why this priority**: Prevents cascading failures; validates isolation design.

**Independent Test**: Configure high error probability for SMS/MMS only; send mixed SMS and Email traffic; observe breaker open events only for sms-mms; email successes unaffected.

**Acceptance Scenarios**:
1. **Given** sms-mms provider configured with high error rate, **When** I send multiple SMS messages, **Then** its breaker transitions to Open after threshold and Email dispatch continues.
2. **Given** sms-mms breaker is Open, **When** I send an Email, **Then** the email provider processes normally (breaker Closed) and metrics reflect separation.

---
### User Story 3 - Deterministic Testing & Observability (Priority: P3)

As a developer writing automated tests, I can set deterministic seeds per provider so outcome sequences (success, rate limited, error, timeout) are reproducible, enabling stable assertions and benchmark comparisons.

**Why this priority**: Enhances test reliability; supports regression and performance baselining.

**Independent Test**: Set fixed seeds for both providers; dispatch N messages of each type; verify expected outcome pattern matches golden test data.

**Acceptance Scenarios**:
1. **Given** seeds configured for both providers, **When** I dispatch 10 SMS and 10 Email messages, **Then** the sequence of outcomes matches the fixed expected pattern.
2. **Given** provider seeds unchanged between runs, **When** I rerun tests, **Then** metrics and logs show identical ordered outcomes for the same message batch.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

- Unknown channel in event payload → dispatch skipped; error logged; counted in invalid_routing metric.
- Provider registry missing entry for a known channel → 500 response for outbound attempt; error surfaced once; event marked processed to avoid infinite retry.
- High sustained error rate for one provider → only that provider’s breaker opens; others unaffected.
- Seed not provided for one provider → falls back to non-deterministic mode while other provider remains deterministic.
- Rapid alternating success/failure causing breaker flapping → enforce minimum half-open sample size to stabilize.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST route outbound SMS and MMS messages via a dedicated sms-mms provider component.
- **FR-002**: System MUST route outbound Email messages via a dedicated email provider component.
- **FR-003**: System MUST maintain an internal provider registry mapping channel → provider and fail fast if lookup is missing.
- **FR-004**: System MUST isolate circuit breaker state per provider; failures for one provider MUST NOT alter another provider’s breaker state.
- **FR-005**: System MUST record provider identity for each dispatch attempt in structured logs and metrics.
- **FR-006**: System MUST support per-provider configurable outcome probabilities (timeout, error, rate_limited) with fallback to global defaults.
- **FR-007**: System MUST expose per-provider counters: attempts, success, rate_limited, error, breaker_transitions.
- **FR-008**: System MUST allow deterministic seeding per provider to reproduce dispatch outcome sequences during tests.
- **FR-009**: System MUST tag stored outbound message records with provider name for later correlation and troubleshooting.
- **FR-010**: System MUST return a clear 500 error when a required provider mapping is missing and log the condition once per occurrence.
- **FR-011**: System MUST treat 429 (rate_limited) outcomes as non-failure events for breaker logic.
- **FR-012**: System MUST provide an invalid_routing metric counter for malformed/unknown channel events.
- **FR-013**: System MUST preserve existing public API endpoints and response semantics (backward compatibility assured).

### Key Entities

- **Outbound Message**: Logical outbound communication: channel (sms|mms|email), from, to, body, attachments, timestamp, idempotency_key?, provider_name (assigned post-routing), status.
- **Provider**: Abstraction representing a dispatch capability for one or more channels; attributes: name, probabilities config (timeout_pct, error_pct, ratelimit_pct), seed (optional), breaker state.
- **Dispatch Attempt**: Outcome record (message_id, provider_name, outcome, latency_ms, error_code?).
- **Provider Registry**: Mapping structure channel → provider reference used during routing.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: 100% of valid outbound messages (SMS, MMS, Email) are routed to a provider with a recorded provider_name field.
- **SC-002**: Breaker isolation: Induced failure test shows 0 unintended breaker state changes in non-target provider (≥ 10 consecutive target failures while other provider remains Closed).
- **SC-003**: Deterministic test run with seeds produces identical ordered outcome sequences across at least 3 consecutive runs.
- **SC-004**: Metrics separation: Per-provider counters reflect distinct counts (no cross-provider contamination) verified by mixed traffic test (≥ 20 messages). 
- **SC-005**: Missing provider mapping scenario returns 500 within < 100 ms and increments a single error metric without duplicate logs.
- **SC-006**: No regression: Existing API success responses (202) for outbound messaging remain unchanged (verified against prior test suite baseline).

### Assumptions

- Global fallback config values remain available; per-provider overrides are optional enhancements.
- No multi-tenancy provider differentiation required in this step.
- Circuit breaker thresholds reuse existing service defaults (not redefined here).
- Deterministic seeding applies only at service start (no hot-reload of seeds).

### Scope Boundaries

- Excludes integration with real external providers.
- Excludes per-customer provider routing rules or dynamic selection logic.
- Excludes UI or dashboard changes; observability limited to logs/metrics.

### Success Validation Approach

Execute automated test suite including seeded outcome tests, isolation failure tests, and mixed traffic metrics verification; review logs for provider_name presence and ensure no NEEDS CLARIFICATION markers remain.
