# Data Model: Initial Schema (PoC)

This documents the intended tables, constraints, triggers, and views implemented by the initial migrations.

## Core Tables

- customers(id PK, name TEXT NOT NULL, created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now())
- providers(id PK, customer_id FK→customers, kind TEXT CHECK (kind IN ('sms','mms','email','voice','voicemail')), name TEXT NOT NULL, credentials_ref TEXT, created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now())
- endpoint_mappings(id PK, customer_id FK→customers, provider_id FK→providers, kind TEXT CHECK (kind IN ('sms','mms','email')), address TEXT NOT NULL, normalized TEXT NOT NULL, hash BIGINT NOT NULL, created_at TIMESTAMPTZ DEFAULT now(), UNIQUE(customer_id, kind, normalized))
- contacts(id PK, customer_id FK→customers, display_name TEXT, created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now())
- conversations(id PK, customer_id FK→customers, topic TEXT NULL, created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now())
- conversation_participants(conversation_id FK→conversations, contact_id FK→contacts, role TEXT CHECK (role IN ('customer','contact')), PRIMARY KEY(conversation_id, contact_id, role))
- messages(id PK, conversation_id FK→conversations, provider_id FK→providers, direction TEXT CHECK (direction IN ('inbound','outbound')), body_id BIGINT NULL, sent_at TIMESTAMPTZ NOT NULL, received_at TIMESTAMPTZ NULL, created_at TIMESTAMPTZ DEFAULT now())
- attachments(id PK, url_id BIGINT NOT NULL, content_type TEXT NULL, created_at TIMESTAMPTZ DEFAULT now())
- message_attachments(message_id FK→messages, attachment_id FK→attachments, PRIMARY KEY(message_id, attachment_id))

Notes:
- `messages.body_id` refers to either `email_bodies.id` (email) or `xms_bodies.id` (sms/mms) by application convention in PoC.

## Deduplication Tables

- email_bodies(id BIGINT PRIMARY KEY, raw TEXT NOT NULL, hash BIGINT UNIQUE NOT NULL, normalized TEXT NOT NULL)
- xms_bodies(id BIGINT PRIMARY KEY, raw TEXT NOT NULL, hash BIGINT UNIQUE NOT NULL, normalized TEXT NOT NULL)
- phone_numbers(id BIGINT PRIMARY KEY, raw TEXT NOT NULL, hash BIGINT UNIQUE NOT NULL, e164 TEXT NOT NULL)
- email_addresses(id BIGINT PRIMARY KEY, raw TEXT NOT NULL, hash BIGINT UNIQUE NOT NULL, lowered TEXT NOT NULL)
- attachment_urls(id BIGINT PRIMARY KEY, raw TEXT NOT NULL, hash BIGINT UNIQUE NOT NULL)

## Indexing (Initial)

- messages(conversation_id, sent_at DESC)
- conversation_participants(contact_id)
- endpoint_mappings(customer_id, kind, hash)
- inbound_events(status, available_at)

## Triggers

- BEFORE INSERT on dedup tables to fill normalized and hash columns:
  - email_bodies: lowercase where applicable, trim, collapse whitespace, Unicode NFC → hash64
  - xms_bodies: trim, collapse whitespace, Unicode NFC → hash64
  - phone_numbers: strip non-digits, best-effort E.164 → hash64
  - email_addresses: lowercase local@domain → hash64
- BEFORE INSERT/UPDATE on tables with `updated_at` to set `updated_at = now()`
- BEFORE INSERT on core timestamped tables to convert inputs to UTC

## Views

- conversation_overview(conversation_id, customer_id, last_message_at, message_count, participant_count)
  - last_message_at = max(messages.sent_at or received_at)
  - message_count = count(*) per conversation
  - participant_count = count(distinct contact_id) from conversation_participants

- conversation_messages(conversation_id, message_id, direction, provider_id, sent_at, received_at, body_text, attachments_count)
  - body_text = NULL in DB view (polymorphic body_id); applications can join to email_bodies/xms_bodies and coalesce normalized text
  - attachments_count derived from message_attachments

## Queue (PoC)

- inbound_events(id PK, event_type TEXT NOT NULL, payload JSONB NOT NULL, received_at TIMESTAMPTZ DEFAULT now(), available_at TIMESTAMPTZ NOT NULL, attempts INT DEFAULT 0, status TEXT CHECK (status IN ('pending','processing','done','dead')))
- Dequeue pattern uses SELECT ... FOR UPDATE SKIP LOCKED with visibility timeout.

## Migration Notes

- Use monotonic timestamps in filenames (sqlx_migrate format)
- Ensure id generation strategy: BIGSERIAL or application-supplied IDs (for dedup tables we expect app-supplied id == hash or a separate ulid; PoC chooses hash-derived id via app)
- All constraints validated; defer heavy indexes if needed for large backfills (out of scope here)

## Sample Inserts (US2)

Example: create a customer, a contact, a conversation with participants, and an outbound message.

```sql
-- Customer
INSERT INTO customers(name) VALUES ('Acme, Inc.') RETURNING id; -- => :customer_id

-- Contact
INSERT INTO contacts(customer_id, display_name)
VALUES (:customer_id, 'Jane Doe') RETURNING id; -- => :contact_id

-- Conversation
INSERT INTO conversations(customer_id, topic)
VALUES (:customer_id, 'Onboarding') RETURNING id; -- => :conversation_id

-- Participants
INSERT INTO conversation_participants(conversation_id, contact_id, role)
VALUES
  (:conversation_id, :contact_id, 'contact'),
  (:conversation_id, :contact_id, 'customer')
ON CONFLICT DO NOTHING;

-- Provider (email example)
INSERT INTO providers(customer_id, kind, name)
VALUES (:customer_id, 'email', 'Sendgrid') RETURNING id; -- => :provider_id

-- Message (ordered by (conversation_id, sent_at DESC) via index)
INSERT INTO messages(conversation_id, provider_id, direction, sent_at)
VALUES (:conversation_id, :provider_id, 'outbound', now());
```
