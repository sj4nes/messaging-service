# Data Model: Unified Messaging

Date: 2025-11-06

## Entities

### Message
- id: string (internal)
- channel: enum (sms|mms|email)
- direction: enum (outbound|inbound)
- from: string (normalized)
- to: string (normalized)
- body: string
- attachments: string[] (URLs)
- timestamp: RFC3339 string (source timestamp)
- processed_at: RFC3339 string (ingest/dispatch time)
- status: enum (pending|dispatched|failed|received)

Validation:
- sms/email body non-empty; mms requires attachments length >= 1.
- timestamp parse or fallback to ingest time.

### Conversation
- id: string (derived key)
- key: tuple (channel, normalized_from, normalized_to) (direction-agnostic ordering rule)
- message_count: number
- last_activity_at: RFC3339 string

### DispatchAttempt
- id: string
- message_id: string
- attempt_number: number
- outcome: enum (success|rate_limited|error)
- latency_ms: number
- error_code: string|null
- occurred_at: RFC3339 string

### ProviderMockConfig
- channel: enum (sms|mms|email)
- success_pct: number (0..100)
- rate_limited_pct: number (0..100)
- error_pct: number (0..100)
- seed: number|null

### CircuitBreakerState
- channel: enum (sms|mms|email)
- state: enum (closed|open|half_open)
- consecutive_failures: number
- last_transition_at: RFC3339 string

## Relationships
- Message 1..* — 0..* DispatchAttempt (per outbound message)
- Conversation 1..* — 0..* Message
- ProviderMockConfig 1 — 0..* Message (by channel)

