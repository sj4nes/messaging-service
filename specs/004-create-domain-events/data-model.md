# Data Model: Domain Events Catalog

This document extracts entities, the common envelope, and event families from the feature spec.

## Entities

- Customer: customer_id, name, status (enabled/disabled), changed_fields?, reason?
- Contact: contact_id, customer_id, display_name?, status, changed_fields?
- Provider: provider_id, customer_id, kind (sms/mms/email/voice/voicemail), status, changed_fields?
- Channel: channel_id, customer_id, kind (sms/mms/email), address, normalized, provider_id, status?, changed_fields?
- Conversation: conversation_id, customer_id, topic?, status (open/closed/archived), participant_ids?, changed_fields?
- EventEnvelope: event_name, event_id, aggregate_type, aggregate_id, occurred_at (UTC ISO-8601), actor, version (int), idempotency_key?

## Event Envelope (common to all)

Required: event_name, event_id, aggregate_type, aggregate_id, occurred_at, actor, version
Optional: idempotency_key

## Event Families & Required Fields

- Customer
  - CustomerCreated: { customer_id, name }
  - CustomerUpdated: { customer_id, changed_fields }
  - CustomerEnabled: { customer_id, reason? }
  - CustomerDisabled: { customer_id, reason? }

- Contact
  - ContactCreated: { contact_id, customer_id, display_name? }
  - ContactUpdated: { contact_id, customer_id, changed_fields }
  - ContactDeleted: { contact_id, customer_id }

- Provider
  - ProviderConfigured: { provider_id, customer_id, kind }
  - ProviderUpdated: { provider_id, customer_id, changed_fields }
  - ProviderEnabled: { provider_id, customer_id }
  - ProviderDisabled: { provider_id, customer_id, reason? }

- Channel
  - ChannelMapped: { channel_id, customer_id, kind, address, normalized, provider_id }
  - ChannelUpdated: { channel_id, customer_id, changed_fields }
  - ChannelUnmapped: { channel_id, customer_id, reason? }

- Conversation
  - ConversationCreated: { conversation_id, customer_id, channel_id, contact_id, topic?, status }
  - ConversationUpdated: { conversation_id, customer_id, changed_fields }
  - ConversationClosed: { conversation_id, customer_id, reason? }
  - ConversationReopened: { conversation_id, customer_id, reason? }
  - ConversationParticipantAdded: { conversation_id, customer_id, participant_id }
  - ConversationParticipantRemoved: { conversation_id, customer_id, participant_id }
  - ConversationArchived: { conversation_id, customer_id, reason? }

## State Transitions (Conversation)

- open → closed (ConversationClosed)
- closed → reopened (ConversationReopened)
- any → archived (ConversationArchived)

## Validation Rules

- occurred_at MUST be UTC ISO-8601 (e.g., 2025-11-05T14:00:00Z)
- event_id MUST be unique (ULID recommended)
- version MUST be >= 1 (int)
- idempotency_key, if present, MUST be stable for a logical event
- Privacy: avoid PII in values; prefer stable identifiers

## Notes & Catalog Alignment

- The detailed purposes, invariants, and example references for each event are maintained in `contracts/events/catalog.md`. This document lists the entities and required field sets; consult the catalog for semantics and example payload filenames.
