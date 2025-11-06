# Event Catalog (004-create-domain-events)

This catalog lists event types, purposes, and required fields (in addition to the common EventEnvelope specified in `envelope.schema.json`). Field names use snake_case.

## Customer
- CustomerCreated: { customer_id, name }
- CustomerUpdated: { customer_id, changed_fields }
- CustomerEnabled: { customer_id, reason? }
- CustomerDisabled: { customer_id, reason? }

## Contact
- ContactCreated: { contact_id, customer_id, display_name? }
- ContactUpdated: { contact_id, customer_id, changed_fields }
- ContactDeleted: { contact_id, customer_id }

## Provider
- ProviderConfigured: { provider_id, customer_id, kind }
- ProviderUpdated: { provider_id, customer_id, changed_fields }
- ProviderEnabled: { provider_id, customer_id }
- ProviderDisabled: { provider_id, customer_id, reason? }

## Channel
- ChannelMapped: { channel_id, customer_id, kind, address, normalized, provider_id }
- ChannelUpdated: { channel_id, customer_id, changed_fields }
- ChannelUnmapped: { channel_id, customer_id, reason? }

## Conversation
- ConversationCreated: { conversation_id, customer_id, channel_id, contact_id, topic?, status }
- ConversationUpdated: { conversation_id, customer_id, changed_fields }
- ConversationClosed: { conversation_id, customer_id, reason? }
- ConversationReopened: { conversation_id, customer_id, reason? }
- ConversationParticipantAdded: { conversation_id, customer_id, participant_id }
- ConversationParticipantRemoved: { conversation_id, customer_id, participant_id }
- ConversationArchived: { conversation_id, customer_id, reason? }
