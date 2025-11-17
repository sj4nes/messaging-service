# Research: Domain Events Catalog

Date: 2025-11-05
Feature: 004-create-domain-events

## Envelope Fields
- Decision: Use envelope fields { event_name, event_id, aggregate_type, aggregate_id, occurred_at (UTC ISO‑8601), actor, version (int), idempotency_key? }
- Rationale: Supports auditability, ordering, and deduplication across transports.
- Alternatives: Expose transport-specific metadata (Kafka offset, HTTP headers) — rejected to keep catalog transport‑agnostic.

## Identifiers
- Decision: event_id as ULID; aggregate_id as stable string/UUID from source system; actor as string.
- Rationale: ULID is sortable by time and globally unique; flexible aggregate id shapes.
- Alternatives: UUID v4 for event_id (acceptable); Snowflake IDs (extra infra).

## Field Naming
- Decision: snake_case for field names; PascalCase for event_name.
- Rationale: Consistent with existing data model docs; readable for multi-language.
- Alternatives: camelCase fields (common in JS) — acceptable but reduces alignment with SQL naming.

## Versioning Policy
- Decision: Backward-compatible additions bump `version`; breaking changes use a new `event_name`.
- Rationale: Consumers can accept newer versions; breakers are explicit.
- Alternatives: Single numeric major version per domain — heavier coordination.

## Privacy
- Decision: No raw PII in broadly broadcast events; use stable identifiers and redaction.
- Rationale: Minimizes leakage risk and compliance overhead.
- Alternatives: Encrypted blobs in events — not needed for catalog.

## Channel Semantics
- Decision: A Channel is a customer-owned address (e.g., phone/email) serviced by a Provider; events describe mapping and updates.
- Rationale: Aligns with domain model and conversation identity.
- Alternatives: Keep the term Endpoint — replaced per clarification.

## Conversation Identity & Lifecycle
- Decision: Identity tuple (customer_id, channel_id, contact_id). States: open, closed, reopened, archived.
- Rationale: Supports grouping and routing; clear lifecycle events.
- Alternatives: Server-generated conversation keys only — less interoperable.
