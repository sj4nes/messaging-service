# Feature Specification: Conversation Persistence & Unification

**Feature Branch**: `009-conversation-persistence`  
**Created**: 2025-11-07  
**Status**: Draft  
**Input**: User description: "Persist and unify conversation grouping so all messages are consistently assigned to a durable conversation keyed by normalized participant tuple + channel; replace placeholder listing with full creation/upsert + indexing; maintain API compatibility."

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

### User Story 1 - Durable, unified conversations (Priority: P1)

As a service owner, I need every inbound and outbound message to be attached to the same durable conversation regardless of direction so reporting, search, and pagination are consistent.

**Why this priority**: This eliminates divergent in-memory vs. persistent behavior and is foundational for analytics and a stable API experience.

**Independent Test**: Send one outbound message and receive one inbound reply between the same two participants on the same channel; verify there is exactly one conversation with both messages attached and correct last-activity timestamp.

**Acceptance Scenarios**:

1. **Given** no existing conversation for a participant pair on a channel, **When** the first message is recorded, **Then** a new conversation is created and the message is linked to it.
2. **Given** an existing conversation for a participant pair on a channel, **When** additional messages are recorded, **Then** those messages are linked to the same conversation and the conversation’s message count and last activity are updated.

---

### User Story 2 - Deterministic conversation listing (Priority: P2)

As an API consumer, I need to list conversations deterministically from the persistent store, with stable ordering and pagination.

**Why this priority**: Users depend on reliable pagination and complete views; temporary empty states or fallbacks cause confusion and missed data.

**Independent Test**: Call the conversation list endpoint repeatedly with the same filters and page parameters and verify the same ordered results are returned from persistence even when in-memory caches are empty.

**Acceptance Scenarios**:

1. **Given** the API is queried for conversations, **When** persistence is available, **Then** the list comes from the persistent store with deterministic ordering and includes participant addresses and a stable conversation key.

---

### User Story 3 - Conversation messages with safe snippets (Priority: P3)

As a support analyst, I need to view messages within a conversation with accurate from/to addresses and a short body snippet that never breaks characters.

**Why this priority**: Enables quick triage while protecting readability and international content.

**Independent Test**: Retrieve messages for a conversation where bodies include multi-byte characters; verify snippets are truncated to the configured length without breaking characters and from/to are correctly presented.

**Acceptance Scenarios**:

1. **Given** conversation messages with long, multi-byte content, **When** retrieving the list, **Then** snippets are truncated at a character boundary to the configured length.
2. **Given** duplicate message submissions (e.g., retried outbound or reprocessed inbound), **When** they are recorded, **Then** duplicates do not inflate the conversation’s message count.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

- Concurrent inserts for the same participant pair and channel (ensure at most one conversation and consistent counts).
- Duplicate message deliveries (idempotency; count remains correct).
- Unicode/emoji message bodies (snippet truncation preserves character boundaries).
- Phone numbers with formatting variations (spaces, dashes, leading zeros) and optional leading "+".
- Persistence temporarily unavailable (API behavior degrades gracefully while still acknowledging message persistence when possible).

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: For every inbound or outbound message recorded, the system MUST derive a canonical conversation key as (channel, participant A, participant B) independent of message direction; participant addresses are normalized consistently per channel and ordered deterministically.
- **FR-002**: The system MUST ensure a durable conversation record exists for the key and link the message to it; if absent, create; if present, reuse.
- **FR-003**: Each message MUST be stored with a non-null reference to its conversation.
- **FR-004**: When recording a message, the conversation’s message count and last activity timestamp MUST be updated atomically with the message so concurrent traffic remains consistent.
- **FR-005**: Conversation listing MUST be sourced from durable storage with deterministic pagination; in-memory data MAY be used only when durable storage is unavailable.
- **FR-006**: Message listing within a conversation MUST include original from/to addresses and a body snippet whose maximum length is configurable (default 64 characters) without breaking characters.
- **FR-007**: Address normalization MUST apply per channel: emails lowercased; phone numbers reduced to digits with optional leading “+”, with spaces/dashes removed; other channels will define their normalizers as they are introduced.
- **FR-008**: Duplicate message submissions MUST NOT increase the conversation’s message count.
- **FR-009**: Provide a deterministic test fixture that proves an outbound + inbound pair creates exactly one conversation for a given channel.
- **FR-010**: A maintenance process MUST backfill conversation assignments for any previously stored messages that lack a conversation reference; the process MUST be idempotent.
- **FR-011**: Counters MUST be exposed for conversations created, conversations reused, and failures to attach a conversation.
- **FR-012**: If conversation assignment fails, the system MUST record the message and mark its status to indicate conversation assignment failed, and log the error with key details.
- **FR-013**: API responses MUST only return a server error when both message persistence and conversation assignment fail; otherwise respond success and record degraded metrics.
- **FR-014**: Normalization and key derivation MUST be side-effect free and covered by unit tests.

Clarifications required (limit 3):

- **FR-015**: Email plus-addressing MUST be treated as equivalent to the base address (e.g., user+tag@example.com is normalized to user@example.com).
- **FR-016**: Channels in scope for this release are Email and SMS/MMS; other channels are explicitly out of scope for this feature.

#### Assumptions

- Initial channels in scope are Email and SMS/MMS; additional channels will adopt the same normalization framework when added.
- Default snippet length is 64 characters; configurable via standard configuration mechanisms.
- For message timelines, the most relevant timestamp per message is used consistently (received time for inbound; sent time for outbound) when both exist.
- Email plus-tag equivalence is applied (user+tag@example.com → user@example.com).

### Key Entities *(include if feature involves data)*

- **Conversation**: A durable grouping representing two normalized participants on a specific channel; attributes include channel, participant A, participant B, message count, created date, and last activity date; identified by a stable key derived from channel + ordered normalized participants.
- **Message**: A unit of communication with direction (inbound/outbound), from/to addresses, body, timestamps, and a required reference to its Conversation.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: A new outbound + inbound pair between the same participants on the same channel yields exactly 1 conversation containing 2 messages, and last activity reflects the most recent message time.
- **SC-002**: Under concurrent submission of 100 messages for the same participant pair and channel, no more than 1 conversation is created and the conversation’s message count equals 100.
- **SC-003**: Conversation assignment (key derivation + create/reuse + link) completes within a local P95 of 10 ms and within 25 ms P95 at 100 requests per second in test conditions.
- **SC-004**: Backfill of a test dataset (10k messages; 1k participant pairs) completes in under 5 seconds locally and produces accurate conversation counts and last activity dates.
- **SC-005**: When conversation assignment is intentionally failed in a test, the message is still persisted with a degraded status and errors are logged; a server error is returned only when both message persistence and conversation assignment fail.
