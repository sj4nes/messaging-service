# Feature Specification: Domain Events for Messaging

**Feature Branch**: `004-create-domain-events`  
**Created**: 2025-11-05  
**Status**: Draft  
**Input**: User description: "Now that we have some persistence, it's probably a good time to define some structs to represent the business events of messaging. Now we are in the process of event storming. We need to create customers. Disable customers. Enable customers. Change customers. Contacts. Endpoints. Providers."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Define canonical event catalog (Priority: P1)

As a platform engineer, I need a canonical, human-readable catalog of messaging domain events (with purpose and required fields) so teams can design, test, and discuss behavior consistently.

**Why this priority**: Establishes shared language; unblocks modeling, contracts, and future automation.

**Independent Test**: Reviewable catalog exists with clear names, descriptions, and example payloads for each event type; stakeholders agree on semantics.

**Acceptance Scenarios**:

1. **Given** the list of core aggregates (Customer, Contact, Provider, Channel), **When** reviewing the catalog, **Then** each lifecycle action is represented by an event type with a clear description and example payload.
2. **Given** two different teams, **When** they describe a "customer disabled" scenario, **Then** both reference the same event name and required fields.

---

### User Story 2 - Define event envelopes and invariants (Priority: P1)

As a data/QA stakeholder, I need event envelope rules (ids, timestamps, actor, idempotency, version) to validate events independently of transport/implementation.

**Why this priority**: Enables testability, auditability, and future transport-agnostic processing.

**Independent Test**: A validator checklist can be applied to any event instance to confirm invariants without code.

**Acceptance Scenarios**:

1. **Given** an event instance, **When** applying the envelope rules, **Then** it contains: event_name, event_id (UUID/ULID), aggregate_type, aggregate_id, occurred_at (UTC), actor, version, and optional idempotency_key.
2. **Given** duplicate deliveries with the same idempotency_key, **When** validated, **Then** they are recognized as the same logical event.

---

### User Story 3 - Model core lifecycle events (Priority: P2)

As a product stakeholder, I need core lifecycle events for Customers, Contacts, Providers, and Endpoints to support audit logs and future automations.

**Why this priority**: Captures critical business transitions and compliance signals.

**Independent Test**: For each lifecycle action, an event definition exists with fields and example.

**Acceptance Scenarios**:

1. Customer: CustomerCreated, CustomerUpdated, CustomerEnabled, CustomerDisabled.
2. Contact: ContactCreated, ContactUpdated, ContactDeleted.
3. Provider: ProviderConfigured, ProviderUpdated, ProviderDisabled, ProviderEnabled.
4. Channel: ChannelMapped, ChannelUnmapped, ChannelUpdated.
5. Conversation: ConversationCreated, ConversationUpdated, ConversationClosed, ConversationReopened, ConversationParticipantAdded, ConversationParticipantRemoved, ConversationArchived.

---

### Edge Cases

- Conflicting updates: two events with different versions target the same aggregate; the higher version wins by policy.
- Late delivery: an older event arrives after a newer one; consumers MUST use version/occurred_at to handle ordering.
- Redelivery: same event delivered multiple times; idempotency_key MUST allow consumers to detect duplicates.
- PII leakage: events MUST avoid sensitive content in public streams; use stable identifiers and redaction.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001 (Catalog)**: Define a canonical list of event types covering lifecycle transitions for Customers, Contacts, Providers, Channels, and Conversations, each with: name, purpose, required fields, and example payload.
- **FR-002 (Envelope)**: All events MUST include an envelope with: event_name, event_id, aggregate_type, aggregate_id, occurred_at (UTC), actor (system/user), version (int), and optional idempotency_key.
- **FR-003 (Semantics)**: Each event MUST state invariants (what changed, what did not), preconditions, and postconditions in business terms.
- **FR-004 (Validation)**: Provide a validation checklist (technology-agnostic) to confirm envelope and required fields for any event instance.
- **FR-005 (Privacy)**: Event definitions MUST avoid sensitive PII in commonly broadcast streams; use stable identifiers and redaction where applicable.
- **FR-006 (Change Policy)**: Document versioning policy for events (backward-compatible changes permitted with version bump; breaking changes require new event names or major version). No NEEDS CLARIFICATION markers remain.
- **FR-007 (Traceability)**: For lifecycle events that correspond to persistence changes, include references (e.g., aggregate_id) sufficient for audit traceability without exposing internal schema.

### Key Entities

- **Customer**: Organization using the platform. Key attributes in events: customer_id, status, name, changed_fields, reason.
- **Contact**: End-user or prospect. Event attributes: contact_id, customer_id, status, changed_fields.
- **Provider**: Messaging provider configuration for a customer. Event attributes: provider_id, customer_id, kind, status, changed_fields.
- **Channel**: Customer-owned channel used to send/receive messages (e.g., phone number, email). Event attributes: channel_id, customer_id, kind, address, normalized, provider_id (servicing provider), action.
- **Event Envelope**: event_name, event_id, aggregate_type, aggregate_id, occurred_at (UTC), actor, version, idempotency_key.
 - **Conversation**: Thread grouping related messages. Event attributes: conversation_id, customer_id, status (open/closed/archived), topic, changed_fields, participant_ids (for add/remove events). Identity note: the tuple (customer_id, channel_id, contact_id) identifies a conversation for routing and grouping in this spec.

### Non-Functional Quality Attributes

(No runtime guarantees in this spec; definitions are transport‑agnostic. Privacy and redaction covered under FR‑005.)
 - **Conversation**: Thread grouping related messages. Event attributes: conversation_id, customer_id, status (open/closed/archived), topic, changed_fields, participant_ids (for add/remove events).

## Clarifications

### Session 2025-11-05

- Q: Should conversation lifecycle and participant-change events be included in the domain event catalog? → A: Yes — add ConversationCreated, ConversationUpdated, ConversationClosed, ConversationReopened, ConversationParticipantAdded, ConversationParticipantRemoved, ConversationArchived.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Event catalog covers ≥ 95% of identified lifecycle transitions from the event-storming workshop (as agreed by stakeholders).
- **SC-002**: 100% of event definitions include envelope, required fields, and example payloads.
- **SC-003**: Validation checklist applied to 10 sampled events yields 0 critical failures (envelope/required fields missing).
- **SC-004**: Stakeholder sign-off achieved (≥ 2 cross-functional reps) with no open clarifications in the spec.

## Clarifications

### Session 2025-11-05

- Q: Should conversation lifecycle and participant-change events be included in the domain event catalog? → A: Yes — add ConversationCreated, ConversationUpdated, ConversationClosed, ConversationReopened, ConversationParticipantAdded, ConversationParticipantRemoved, ConversationArchived.
- Q: Rename "endpoint" events to "channel" events and define channel semantics? → A: Yes — replace EndpointMapped/EndpointUnmapped/EndpointUpdated with ChannelMapped/ChannelUnmapped/ChannelUpdated; a Channel is a customer-owned address (phone/email) serviced by a Provider; conversation identity uses (customer_id, channel_id, contact_id); conversations can be open, resumed, or closed as messages occur.
