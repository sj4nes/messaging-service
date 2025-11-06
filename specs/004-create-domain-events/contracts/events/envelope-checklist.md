# Envelope Checklist (FR-002)

Use this checklist to validate any event instance, independently of transport.

Required envelope fields (see `envelope.schema.json`):

- [ ] event_name — non-empty string (PascalCase)
- [ ] event_id — globally unique (ULID/UUID) string
- [ ] aggregate_type — one of: Customer | Contact | Provider | Channel | Conversation
- [ ] aggregate_id — non-empty stable identifier string
- [ ] occurred_at — ISO 8601 UTC timestamp (e.g., 2025-11-05T14:00:00Z)
- [ ] actor — non-empty string (e.g., system or admin:user_42)
- [ ] version — integer >= 1
- [ ] idempotency_key — OPTIONAL: present when deduplication across deliveries is required

Additional guidance:

- [ ] Names and field casing follow spec: PascalCase for event_name; snake_case for fields
- [ ] No PII: avoid sensitive values; prefer stable identifiers
- [ ] For duplicate deliveries: same idempotency_key SHOULD be used
- [ ] For ordering: consumers SHOULD use occurred_at and/or version

Mapping to FR-002:

| FR-002 Field      | Check                                   | Pass Criteria                              |
|-------------------|-----------------------------------------|--------------------------------------------|
| event_name        | Present, non-empty                      | String length >= 1                          |
| event_id          | Present, unique                         | String length >= 8                          |
| aggregate_type    | Present, valid enum                     | One of [Customer, Contact, Provider, Channel, Conversation] |
| aggregate_id      | Present, non-empty                      | String length >= 1                          |
| occurred_at       | Present, ISO-8601 UTC                   | Matches JSON Schema date-time format        |
| actor             | Present, non-empty                      | String length >= 1                          |
| version           | Present, integer >= 1                   | Integer and >= 1                            |
| idempotency_key   | Optional but stable if present          | String length >= 4                          |
