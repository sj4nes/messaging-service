# Feature Spec: PoC SQLx Migrations and DB Schema

Feature: 003-poc-sqlx-migration
Date: 2025-11-05

## Summary

Introduce an initial PostgreSQL schema, migrations, and an admin utility (renamed to `db-migrate`) to manage schema evolution using SQLx. The scope includes normalized entities for Customers, Providers, Contacts, Conversations, Messages, Attachments, and mapping tables for provider endpoints. Implement deduplication for email addresses, phone numbers, message bodies, and attachment URLs via application-level fast hashing and database uniqueness. Define views for conversation lists and conversation messages. Ensure timestamps are stored as UTC. Propose triggers to improve resilience and data hygiene. Consider a PoC queue using PostgreSQL to decouple webhook ingestion from processing.

## Clarifications

### Session 2025-11-05

- Q: Add a dedicated dedup table for SMS/MMS message bodies with similar triggers? → A: Yes — introduce `xms_bodies` for SMS/MMS, with normalization and hashing triggers parallel to `email_bodies`.

## User Scenarios & Testing

- As an operator, I can run a migration command to create the initial schema and roll it forward.
- As a developer, I can insert messages with participants across multiple providers and have them grouped into conversations.
- As an analyst, I can query a "conversation overview" view to see last activity, message counts, and participants.
- As an integrator, I can rely on dedup tables to avoid duplicate emails/phones/bodies/attachment URLs.
- As a backend service, I can insert webhook payloads into a queue table quickly, and a worker can dequeue and process them later.

Acceptance checks:
- Running the migration creates all tables, views, indexes, triggers without errors.
- Uniqueness constraints reject duplicate email/phone/body/url entries (case/format normalized).
- Views return expected shapes for sample data (happy path + dedup edge cases).
- Timestamps are stored consistently as UTC (TIMESTAMPTZ), regardless of input timezone.

## Functional Requirements

1) Migrations and Admin Utility
- Provide a `db-migrate` binary (renamed from `admin`) to run SQLx migrations (applying and creating).
- Store migrations under a migrations directory consumable by `sqlx::migrate!`.
- Exit non-zero on migration errors; print clear diagnostics.

2) Core Entities (PostgreSQL)
- customers(id, name, created_at, updated_at)
- providers(id, customer_id, kind ENUM('sms','mms','email','voice','voicemail'), name, credentials_ref TEXT, created_at, updated_at)
- endpoint_mappings(id, customer_id, provider_id, kind ENUM('sms','mms','email'), address TEXT, normalized TEXT, hash BIGINT, created_at, UNIQUE(customer_id, kind, normalized))
- contacts(id, customer_id, display_name, created_at, updated_at)
- conversations(id, customer_id, topic TEXT NULL, created_at, updated_at)
- conversation_participants(conversation_id, contact_id, role ENUM('customer','contact'), PRIMARY KEY(conversation_id, contact_id, role))
- messages(id, conversation_id, provider_id, direction ENUM('inbound','outbound'), body_id BIGINT NULL, sent_at TIMESTAMPTZ, received_at TIMESTAMPTZ NULL, created_at TIMESTAMPTZ DEFAULT now())
- attachments(id, url_id BIGINT, content_type TEXT NULL, created_at)
- message_attachments(message_id, attachment_id, PRIMARY KEY(message_id, attachment_id))

3) Deduplication Tables
- email_bodies(id BIGINT PRIMARY KEY, raw TEXT, hash BIGINT UNIQUE, normalized TEXT) — for email message body text
- xms_bodies(id BIGINT PRIMARY KEY, raw TEXT, hash BIGINT UNIQUE, normalized TEXT) — for SMS/MMS message body text
- phone_numbers(id BIGINT PRIMARY KEY, raw TEXT, hash BIGINT UNIQUE, e164 TEXT)
- email_addresses(id BIGINT PRIMARY KEY, raw TEXT, hash BIGINT UNIQUE, lowered TEXT)
- attachment_urls(id BIGINT PRIMARY KEY, raw TEXT, hash BIGINT UNIQUE)
- Store application-computed 64-bit fast hashes (e.g., `fasthash`) in `hash` columns; enforce `UNIQUE(hash)`.
- For textual content: email messages use `email_bodies`; SMS/MMS messages use `xms_bodies`. The `messages` row carries a `body_id` that logically refers to the appropriate table based on message kind; referential linkage is enforced by application logic in this PoC. Binary attachments captured via `attachments`/`attachment_urls`.

4) Views
- conversation_overview: conversation_id, customer_id, last_message_at, message_count, participant_count
- conversation_messages: conversation_id, message_id, direction, provider_id, sent_at, received_at, body_text, attachments_count

5) Timestamps & Timezones
- All timestamps persisted as TIMESTAMPTZ; inputs are converted to UTC at insert time.
- Provide triggers to normalize timestamps and set `updated_at` on changes.

6) Triggers & Constraints
- BEFORE INSERT trigger to normalize emails (lowercase), phone numbers (E.164), and compute hashes for dedup tables.
- Provide analogous BEFORE INSERT triggers for `xms_bodies` (e.g., trim/whitespace normalization, Unicode NFC, optional length guard) and for `email_bodies` (case-normalization if applicable) to compute hashes consistently.
- BEFORE INSERT/UPDATE trigger on tables with `updated_at` to maintain audit fields.
- Consider deferred constraints for batched inserts to reduce contention.

7) Queue (PoC)
- inbound_events(id, event_type, payload JSONB, received_at TIMESTAMPTZ DEFAULT now(), available_at TIMESTAMPTZ, attempts INT DEFAULT 0, status ENUM('pending','processing','done','dead'))
- Dequeue pattern using `FOR UPDATE SKIP LOCKED` and a small visibility timeout.
- Webhook handlers only insert into `inbound_events`; a worker drains and processes.

## Assumptions
- Deduplication hashing uses an application-level fast non-cryptographic 64-bit hash; collisions are extremely rare and acceptable for dedup keys at this stage.
- Phone normalization uses E.164; emails normalized to lowercase local+domain.
- Provider credentials are referenced indirectly (e.g., secret store key) rather than stored in-line.
- ENUMs may be represented via PostgreSQL native enums or constrained TEXT + CHECKs—implementation choice is left for planning.

## Success Criteria
- Running `db-migrate` applies migrations successfully on a clean database in < 5 seconds.
- Inserting duplicate emails/phones/bodies/attachment URLs results in a single logical stored record (UNIQUE constraint on hash) with up to 0.1% allowable collision risk.
- Views `conversation_overview` and `conversation_messages` return within 50 ms on a dataset of 10k messages on local dev hardware.
- Webhook ingest (insert into inbound_events) completes in < 30 ms on local dev hardware; worker can dequeue at >= 500 msg/sec sustained in a PoC.

## Non-Goals (for this feature)
- Full REST endpoint implementation for CRUD of these entities.
- Provider-specific credential storage or secret rotation.
- Background worker binary—PoC describes schema and patterns; worker wiring is later.

