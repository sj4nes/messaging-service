# Event Catalog (004-create-domain-events)

This catalog lists event types, purposes, and required fields, in addition to the common EventEnvelope specified in `envelope.schema.json`. Field names use snake_case. Each definition also lists invariants (what changed and what did not), preconditions, postconditions, and an example reference when available.

Tip: Envelope fields are always required: event_name, event_id, aggregate_type, aggregate_id, occurred_at (UTC), actor, version, idempotency_key? (optional).

## Customer

### CustomerCreated
- Purpose: A new customer organization has been created and is available to configure.
- Required: { customer_id, name }
- Invariants: No status change implied beyond initial/default; no contacts or channels created by this event.
- Preconditions: Unique customer_id.
- Postconditions: Customer exists.
- Example: `examples/customer_created.example.json`

### CustomerUpdated
- Purpose: Customer metadata changed (e.g., name).
- Required: { customer_id, changed_fields }
- Invariants: Identity (customer_id) unchanged; status unchanged by this event.
- Preconditions: Customer exists.
- Postconditions: Customer fields reflect updates.

### CustomerEnabled / CustomerDisabled
- Purpose: Operational status toggled for a customer.
- Required: { customer_id, reason? }
- Invariants: Identity and non-status fields unchanged.
- Preconditions: Customer exists; current status differs.
- Postconditions: Status becomes enabled/disabled accordingly.

## Contact

### ContactCreated
- Purpose: A contact is created under a customer.
- Required: { contact_id, customer_id, display_name? }
- Invariants: No channel mappings or conversations implied.
- Preconditions: Customer exists; contact_id unique under customer.
- Postconditions: Contact exists.
- Example: `examples/contact_created.example.json`

### ContactUpdated
- Purpose: Contact metadata changed.
- Required: { contact_id, customer_id, changed_fields }
- Invariants: Identity unchanged; customer_id unchanged.
- Preconditions: Contact exists.
- Postconditions: Contact fields updated.

### ContactDeleted
- Purpose: Contact removed (soft/hard policy out of scope for this catalog).
- Required: { contact_id, customer_id }
- Invariants: N/A
- Preconditions: Contact exists.
- Postconditions: Contact no longer active/available.

## Provider

### ProviderConfigured
- Purpose: A messaging provider configuration is created for a customer.
- Required: { provider_id, customer_id, kind }
- Invariants: No channel mappings implied.
- Preconditions: Customer exists; provider_id unique under customer.
- Postconditions: Provider config exists.
- Example: `examples/provider_configured.example.json`

### ProviderUpdated
- Purpose: Provider configuration updated.
- Required: { provider_id, customer_id, changed_fields }
- Invariants: Identity unchanged.
- Preconditions: Provider exists.
- Postconditions: Fields reflect updates.

### ProviderEnabled / ProviderDisabled
- Purpose: Operational status toggled for a provider configuration.
- Required: { provider_id, customer_id, reason? }
- Invariants: Identity and non-status fields unchanged.
- Preconditions: Provider exists; current status differs.
- Postconditions: Status becomes enabled/disabled accordingly.

## Channel

### ChannelMapped
- Purpose: A customer-owned address (phone/email) is mapped to a provider for messaging.
- Required: { channel_id, customer_id, kind, address, normalized, provider_id }
- Invariants: Channel identity unchanged; mapping created/confirmed.
- Preconditions: Customer and provider exist; channel_id unique under customer.
- Postconditions: Channel mapping is active.
- Example: `examples/channel_mapped.example.json`

### ChannelUpdated
- Purpose: Channel metadata updated (e.g., display label or normalization details).
- Required: { channel_id, customer_id, changed_fields }
- Invariants: Mapping to provider unchanged unless explicitly part of changed_fields.
- Preconditions: Channel exists.
- Postconditions: Channel fields updated.

### ChannelUnmapped
- Purpose: A channel is unmapped from its provider.
- Required: { channel_id, customer_id, reason? }
- Invariants: Channel identity unchanged.
- Preconditions: Channel exists.
- Postconditions: Mapping is inactive/removed.

## Conversation

Notes: Conversation identity for routing/grouping uses the tuple (customer_id, channel_id, contact_id).

### ConversationCreated
- Purpose: A conversation thread is created or resumed on first message or explicit action.
- Required: { conversation_id, customer_id, channel_id, contact_id, topic?, status }
- Invariants: Participants beyond contact/customer/channel not implied.
- Preconditions: Customer, channel, and contact exist.
- Postconditions: Conversation exists in the specified status (typically open).

### ConversationUpdated
- Purpose: Conversation metadata changed (e.g., topic).
- Required: { conversation_id, customer_id, changed_fields }
- Invariants: Identity unchanged; status unchanged by this event.
- Preconditions: Conversation exists.
- Postconditions: Conversation fields updated.

### ConversationClosed / ConversationReopened
- Purpose: Close or reopen a conversation.
- Required: { conversation_id, customer_id, reason? }
- Invariants: Identity and non-status fields unchanged.
- Preconditions: Conversation exists; current status differs.
- Postconditions: Status becomes closed/reopened accordingly.

### ConversationParticipantAdded / ConversationParticipantRemoved
- Purpose: Modify conversation participants (e.g., add/remove an agent or user).
- Required: { conversation_id, customer_id, participant_id }
- Invariants: Conversation identity unchanged.
- Preconditions: Conversation exists.
- Postconditions: Participant set updated.

### ConversationArchived
- Purpose: Move a conversation to archived state.
- Required: { conversation_id, customer_id, reason? }
- Invariants: Identity unchanged.
- Preconditions: Conversation exists.
- Postconditions: Status becomes archived; new activity may create/reopen per business rules (out of scope).
