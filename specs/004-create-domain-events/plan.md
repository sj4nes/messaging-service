# Implementation Plan: 004-create-domain-events

**Branch**: `004-create-domain-events` | **Date**: 2025-11-05 | **Spec**: specs/004-create-domain-events/spec.md

## Summary

Create a clear, transport-agnostic domain event catalog for messaging. Define a canonical event envelope (ids, timestamps, actor, version, idempotency), a set of lifecycle events for Customers, Contacts, Providers, Channels (renamed from Endpoints), and Conversations, plus a validation checklist and examples. No runtime service changes are required; deliver artifacts as Markdown and JSON Schemas with sample payloads.

## Technical Context

**Language/Version**: Rust (repo), Markdown + JSON Schema (this feature)
**Primary Dependencies**: python-jsonschema (local validator), optional AJV for consumers
**Storage**: N/A
- Testing: Validate example payloads against JSON Schemas; lint Markdown
- Target Platform: Dev docs and schema consumers (any language)
**Project Type**: Spec + contracts (no service runtime)
- Performance Goals: N/A (non-runtime)
- Constraints:
  - All timestamps are UTC in ISO 8601 format
  - Avoid PII in broadly shared events (stick to stable identifiers); redaction guidance included
  - Event naming stable; versioning policy defined; idempotency supported
  - Conversation identity uses (customer_id, channel_id, contact_id)

## Constitution Check

- Security-First: No secrets/PII in events; redaction guidance provided. PASS
- Test-First & Quality Gates: Examples validate against JSON Schema; docs lintable. PASS
- Observability: Out of scope for runtime; catalog supports auditability via envelope fields. PASS
- Versioning & Change Control: Event versioning policy defined; changes tracked in docs. PASS
- Simplicity & SRP: Documentation-only feature; no code coupling. PASS

## Project Structure (this feature)

```
specs/004-create-domain-events/
├── plan.md          # This file
├── research.md      # Phase 0 decisions & rationale
├── data-model.md    # Entities, envelope, event fields, lifecycle
├── quickstart.md    # How to use schemas, validate examples
├── contracts/
│   └── events/
│       ├── envelope.schema.json
│       ├── catalog.md
│       └── examples/
│           ├── customer_created.example.json
│           ├── conversation_closed.example.json
│           └── channel_mapped.example.json
└── checklists/
    └── requirements.md
```

## Phase 0: Outline & Research

- Decisions to document in research.md:
  - Envelope fields and types (event_id ULID, occurred_at ISO-8601 UTC, actor string, version int, idempotency_key optional)
  - Naming/casing conventions for fields (snake_case)
  - Event naming conventions (PascalCase with domain prefix)
  - Versioning policy (minor compatible, new name for breaking)
  - Privacy policy (no PII, stable identifiers)
  - Conversation identity tuple and state transitions (open, closed, reopened, archived)
  - Channel semantics (customer-owned address/number/email, serviced by a provider)
- Output: research.md with Decision, Rationale, Alternatives for each item

## Phase 1: Design & Contracts

Prerequisite: research.md complete

1. data-model.md
   - Entities: Customer, Contact, Provider, Channel, Conversation, EventEnvelope
   - For each event family, list required fields and semantics
   - State transitions for Conversation (open/closed/reopened/archived)

2. contracts/events
   - envelope.schema.json (JSON Schema Draft-07)
   - catalog.md listing event names, purposes, and required fields
   - examples/*.example.json validated against envelope.schema.json + per-event required fields
   - contracts/README.md explaining HTTP APIs are out of scope for this feature

3. quickstart.md
   - How to validate example payloads locally using a JSON Schema validator
   - How to extend the catalog with new events while following versioning and privacy rules

4. Agent context update
   - Run `.specify/scripts/bash/update-agent-context.sh copilot` to record the added artifacts for this feature in the agent context

## Phase 2: Post-design Constitution Re-check

- Confirm privacy guidance present; versioning policy explicit; examples validate. Expect PASS.

## Acceptance of Planning Output

- research.md present with all decisions
- data-model.md enumerates entities, events, and states
- contracts/events/ contains envelope schema, catalog.md, and example payloads
- quickstart.md describes validation steps
- Agent context updated
