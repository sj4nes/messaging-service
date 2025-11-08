# Data Model: Conversation Persistence & Unification

## Entities

### Conversation
- id: bigint (PK)
- channel: text (NOT NULL)
- participant_a: text (NOT NULL) — normalized, lexicographically smaller of the two
- participant_b: text (NOT NULL) — normalized, lexicographically larger of the two
- message_count: integer (NOT NULL, default 0)
- last_activity_at: timestamptz (NOT NULL)
- created_at: timestamptz (NOT NULL, default NOW())
- key: text (optional stored/generated) — format: `{channel}:{participant_a}<->{participant_b}`

Constraints/Indexes:
- UNIQUE (channel, participant_a, participant_b)
- INDEX (last_activity_at DESC)

Validation Rules:
- channel in {email, sms, mms}
- participant addresses must be normalized according to channel rules

### Message (changes)
- conversation_id: bigint (NOT NULL, FK → conversations.id)
- direction: enum/text in {Inbound, Outbound}

Validation Rules:
- Non-null conversation_id at insert time
- direction must be one of allowed values

## Normalization Rules

- Email: lowercase entire address; normalize plus-addressing (user+tag@example.com → user@example.com)
- Phone (SMS/MMS): keep optional leading "+"; strip all non-digits except leading plus; remove spaces/dashes; do not change country code
- Ordering: compare normalized_a vs normalized_b; smaller becomes participant_a

## Derived Fields

- Conversation.key derives from `(channel, participant_a, participant_b)`
- Snippet (not stored) computed on read using configured char length, truncating on UTF-8 boundaries

## State Transitions

- Message Insert → Upsert Conversation (create if absent) → Link message with FK → Update conversation.message_count and last_activity_at atomically
- Backfill → For each message with NULL conversation_id, compute key → upsert conversation → set FK → recompute aggregates (single-pass)

## Notes

- This model is additive and backward-compatible with existing APIs; DTOs are extended to include participants and key where applicable.
