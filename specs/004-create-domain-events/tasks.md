---

description: "Tasks to implement Feature 004: create-domain-events"
---

# Tasks: 004-create-domain-events

**Input**: Design documents from `specs/004-create-domain-events/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

Notes:
- Tests are OPTIONAL for this feature; we only validate JSON examples against the envelope schema and lint docs.
- Tasks are grouped by user story to enable independent delivery and review.

## Format: `[ID] [P?] [Story] Description`

- [P]: Can run in parallel (different files, no dependencies)
- [Story]: User story label (US1, US2, US3)
- Each task includes an exact file path

---

## Phase 1: Setup (Shared Infrastructure)

Purpose: Create any missing scaffolding files for contracts docs.

- [X] T001 Create contracts scope notice in specs/004-create-domain-events/contracts/README.md

---

## Phase 2: Foundational (Blocking Prerequisites)

Purpose: Establish basic validation tooling to keep examples and schema consistent.

⚠️ CRITICAL: Complete before user stories begin.

- [X] T002 Add simple validator script in specs/004-create-domain-events/contracts/events/validate_examples.py to validate examples against envelope.schema.json
- [X] T003 [P] Update specs/004-create-domain-events/quickstart.md with a section showing how to run contracts/events/validate_examples.py
- [X] T004 [P] Verify specs/004-create-domain-events/contracts/events/envelope.schema.json conforms to JSON Schema Draft-07 (adjust only if out-of-sync)

Checkpoint: Validation scaffolding ready — user stories can proceed in parallel.

---

## Phase 3: User Story 1 - Canonical event catalog (Priority: P1) 🎯 MVP

Goal: Provide a human-readable catalog of messaging domain events (purpose + required fields) with exemplar examples to establish shared language.

Independent Test: Catalog is reviewable; each aggregate has event entries and at least one example payload.

### Implementation for User Story 1

- [X] T005 [US1] Complete event catalog structure and entries in specs/004-create-domain-events/contracts/events/catalog.md for Customers, Contacts, Providers, Channels, Conversations (names, purposes, required fields)
- [X] T006 [P] [US1] Align specs/004-create-domain-events/data-model.md event families and required fields with contracts/events/catalog.md
- [X] T007 [P] [US1] Add exemplar examples for missing aggregates:
  - specs/004-create-domain-events/contracts/events/examples/provider_configured.example.json
  - specs/004-create-domain-events/contracts/events/examples/contact_created.example.json

Checkpoint: Catalog established with exemplars; teams can reference consistent names and fields.

---

## Phase 4: User Story 2 - Event envelope and invariants (Priority: P1)

Goal: Define envelope rules (ids, timestamps, actor, idempotency, version) and a validation checklist that can be applied to any event instance.

Independent Test: Any example validates against the envelope schema; the checklist covers all FR-002 fields.

### Implementation for User Story 2

- [X] T008 [US2] Finalize envelope fields in specs/004-create-domain-events/contracts/events/envelope.schema.json (event_name, event_id, aggregate_type, aggregate_id, occurred_at, actor, version, idempotency_key?)
- [X] T009 [P] [US2] Create human checklist in specs/004-create-domain-events/contracts/events/envelope-checklist.md mapping FR-002 → observable fields
- [X] T010 [P] [US2] Demonstrate idempotency by adding idempotency_key to one existing example in specs/004-create-domain-events/contracts/events/examples/channel_mapped.example.json

Checkpoint: Envelope invariants are explicit and enforceable via schema + checklist.

---

## Phase 5: User Story 3 - Model core lifecycle events (Priority: P2)

Goal: Provide full lifecycle coverage with detailed required fields and complete examples for Customers, Contacts, Providers, Channels, and Conversations.

Independent Test: For each lifecycle event in the spec, the catalog lists required fields and an example exists.

### Implementation for User Story 3

- [X] T011 [US3] Expand specs/004-create-domain-events/contracts/events/catalog.md with detailed required fields and invariants for all events (Customer*, Contact*, Provider*, Channel*, Conversation*)
- [X] T012 [P] [US3] Add missing Customer examples:
  - specs/004-create-domain-events/contracts/events/examples/customer_updated.example.json
  - specs/004-create-domain-events/contracts/events/examples/customer_enabled.example.json
  - specs/004-create-domain-events/contracts/events/examples/customer_disabled.example.json
- [X] T013 [P] [US3] Add remaining Contact examples:
  - specs/004-create-domain-events/contracts/events/examples/contact_updated.example.json
  - specs/004-create-domain-events/contracts/events/examples/contact_deleted.example.json
- [X] T014 [P] [US3] Add remaining Provider examples:
  - specs/004-create-domain-events/contracts/events/examples/provider_updated.example.json
  - specs/004-create-domain-events/contracts/events/examples/provider_enabled.example.json
  - specs/004-create-domain-events/contracts/events/examples/provider_disabled.example.json
- [X] T015 [P] [US3] Add remaining Channel examples:
  - specs/004-create-domain-events/contracts/events/examples/channel_updated.example.json
  - specs/004-create-domain-events/contracts/events/examples/channel_unmapped.example.json
- [X] T016 [P] [US3] Add remaining Conversation examples:
  - specs/004-create-domain-events/contracts/events/examples/conversation_created.example.json
  - specs/004-create-domain-events/contracts/events/examples/conversation_updated.example.json
  - specs/004-create-domain-events/contracts/events/examples/conversation_reopened.example.json
  - specs/004-create-domain-events/contracts/events/examples/conversation_participant_added.example.json
  - specs/004-create-domain-events/contracts/events/examples/conversation_participant_removed.example.json
  - specs/004-create-domain-events/contracts/events/examples/conversation_archived.example.json
- [X] T017 [US3] Cross-validate specs/004-create-domain-events/data-model.md, contracts/events/catalog.md, and all examples for field parity and semantics; update data-model.md if mismatches exist

Checkpoint: Full coverage complete — every lifecycle event has fields defined and a validating example.

---

## Phase 6: Polish & Cross-Cutting

Purpose: Final verification, documentation polish, and bookkeeping.

- [ ] T018 [P] Run specs/004-create-domain-events/contracts/events/validate_examples.py and resolve any schema validation failures
- [ ] T019 Update CHANGELOG.md with Feature 004 completion summary
- [ ] T020 [P] Update agent context by running .specify/scripts/bash/update-agent-context.sh (target: copilot) to record new artifacts
- [ ] T021 [P] Lint and edit for clarity across specs/004-create-domain-events/ (catalog.md, data-model.md, quickstart.md, research.md)

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1): No dependencies
- Foundational (Phase 2): Depends on Setup; blocks all user stories
- User Stories (Phase 3+): Depend on Foundational; US1 and US2 (both P1) can proceed in parallel; US3 (P2) depends on US1+US2 outputs
- Polish (Final): Depends on all desired stories being complete

### User Story Dependencies

- User Story 1 (P1): Starts after Phase 2; no dependency on other stories
- User Story 2 (P1): Starts after Phase 2; no dependency on other stories
- User Story 3 (P2): Starts after Phase 2; depends on US1 catalog structure and US2 envelope rules

### Within Each User Story

- If writing tests/validators, create them first and ensure they fail before fixes
- Update catalog before adding examples for that story
- Validate examples against the envelope schema

### Parallel Opportunities

- Foundational: T003 and T004 can run in parallel after T002
- US1: T006 and T007 can run in parallel; exemplar examples (T007) can be created in parallel
- US2: T009 and T010 can run in parallel
- US3: T012–T016 can run fully in parallel (each writes distinct files)
- Polish: T018, T020, T021 can run in parallel

---

## Parallel Example: User Story 3

```bash
# In parallel, author independent example files (distinct paths):
# Customer and Contact examples
Task: "Add missing Customer examples" → customer_updated.example.json, customer_enabled.example.json, customer_disabled.example.json
Task: "Add remaining Contact examples" → contact_updated.example.json, contact_deleted.example.json

# Provider, Channel, and Conversation examples
Task: "Add remaining Provider examples" → provider_updated.example.json, provider_enabled.example.json, provider_disabled.example.json
Task: "Add remaining Channel examples" → channel_updated.example.json, channel_unmapped.example.json
Task: "Add remaining Conversation examples" → conversation_created/updated/reopened/participant_added/participant_removed/archived.example.json
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (catalog + exemplars)
4. Stop and validate with stakeholders

### Incremental Delivery

1. US1 (catalog) → validate
2. US2 (envelope invariants + checklist) → validate
3. US3 (full lifecycle examples and details) → validate
4. Polish and record in CHANGELOG.md; update agent context
