# Data Model: Outbound Message Event and Persistence Mapping

## Event: OutboundMessageEvent (schema_version: 1)

Fields:
- schema_version: integer (>=1)
- channel: enum ["sms", "email"]
- customer_id: uuid (FK to customers.id)
- from: string (phone or email)
- to: string (phone or email)
- subject: string | null (email only)
- body: string (non-empty)
- body_hash: string (hex-encoded SHA-256 of body) — optional in request; computed by worker if absent
- sent_at: RFC3339 timestamp — optional; if absent/invalid, worker uses server time
- idempotency_key: string — optional; computed deterministically by worker if absent
- metadata: object (free-form key/value for provider hints)

## Idempotency Key Derivation

key = sha256_hex(
  concat(channel, "|", customer_id, "|", from, "|", to, "|", normalize_ts(sent_at), "|", sha256(body))
)

Normalization: sent_at rounded or truncated to second precision to match DB semantics.

## Persistence Mapping (PostgreSQL)

- message_bodies (deduplicated by hash)
- conversations (upsert by participants + customer_id)
- messages (idempotent insert by idempotency_key)

The worker performs the following transactionally:
1. Insert or select message_bodies by body_hash.
2. Upsert conversation for (customer_id, channel, to, from) with updated timestamps/counters.
3. Insert message with idempotency_key; on conflict do nothing (idempotent), but still ensure conversation updated.

## Error Handling & Retries

- Validation errors: rejected at handler; not enqueued.
- Transient DB errors: retried with exponential backoff (see spec FR-009 Option B).
- Persistent failures (>72h or >10 attempts): moved to DLQ.

## Metrics Mapping

- queue_depth gauge reflects pending events (in-memory size)
- processing_latency histogram from dequeue to persistence commit
- counters for processed/failed/retried
