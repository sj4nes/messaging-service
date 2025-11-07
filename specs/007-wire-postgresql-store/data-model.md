# Data Model: Wire PostgreSQL Store

Created: 2025-11-06

## Entities

### InboundEvent
- id: uuid, pk, default gen_random_uuid()
- channel: text (enum: sms, mms, email)
- from: text (normalized for channel)
- to: text (normalized for channel)
- provider_message_id: text
- payload: jsonb (original provider payload)
- status: text (enum: received, processing, processed, error, dead_letter)
- attempt_count: int (default 0)
- next_attempt_at: timestamptz (nullable)
- processor_id: text (nullable)
- error_code: text (nullable)
- error_message: text (nullable)
- created_at: timestamptz default now()
- updated_at: timestamptz default now()
- processed_at: timestamptz (nullable)

Indexes/Constraints:
- unique(channel, provider_message_id)
- index(status, next_attempt_at)
- index(created_at)

### Message
- id: uuid, pk, default gen_random_uuid()
- inbound_event_id: uuid (nullable for outbound), fk -> InboundEvent.id
- channel: text (sms, mms, email)
- from: text
- to: text
- body: text
- attachments: jsonb (array of strings)
- timestamp: timestamptz
- conversation_key: text (computed from normalized from/to + channel; stored)
- created_at: timestamptz default now()

Indexes/Constraints:
- index(conversation_key, timestamp desc)
- index(timestamp)

### Conversation
- id: uuid, pk, default gen_random_uuid()
- key: text (channel/from/to normalized in lexical order)
- message_count: int
- last_activity_at: timestamptz

Indexes/Constraints:
- unique(key)
- index(last_activity_at desc)

## Validation Rules

- provider_message_id must be non-empty for inbound SMS/MMS/Email
- attachments: null for sms; non-empty for mms; array for email (may be empty)
- body: non-empty for sms/email; mms may have text plus attachments
- timestamp: required ISO 8601

## State Transitions (InboundEvent)

- received -> processing (claimed by worker)
- processing -> processed (success)
- processing -> error (failure; attempt_count++, next_attempt_at set per backoff)
- error -> processing (retry after next_attempt_at)
- error -> dead_letter (when attempt_count > max_retries)
