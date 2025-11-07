# Data Model: Provider Routing By Channel

Date: 2025-11-07

## Entities

### OutboundMessage (internal)
- channel: enum (sms | mms | email)
- from: string
- to: string
- body: string
- attachments: string[] (optional)
- timestamp: string (RFC3339)
- idempotency_key: string (optional)
- provider_name: string (assigned post-routing)
- status: enum (pending | dispatched | failed)

### Provider (logical)
- name: string (e.g., "sms-mms", "email")
- config: { timeout_pct: u32, error_pct: u32, ratelimit_pct: u32, seed?: u64 }
- breaker_state: enum (closed | open | half_open) [derived]

### ProviderRegistry
- mapping: { sms: ProviderRef, mms: ProviderRef, email: ProviderRef }

### DispatchAttempt
- message_id: string/id
- provider_name: string
- outcome: enum (success | rate_limited | error | timeout)
- latency_ms: number
- error_code: string (optional)
- occurred_at: string (RFC3339)

## Validation Rules
- channel must be one of sms|mms|email
- mms requires attachments length >= 1
- body non-empty for sms/email

## Relationships
- ProviderRegistry references Providers
- DispatchAttempt references OutboundMessage
