# Quickstart: Provider Routing By Channel

## Configure Providers

Environment variables (optional overrides):
- API_PROVIDER_SMS_TIMEOUT_PCT
- API_PROVIDER_SMS_ERROR_PCT
- API_PROVIDER_SMS_RATELIMIT_PCT
- API_PROVIDER_SMS_SEED
- API_PROVIDER_EMAIL_TIMEOUT_PCT
- API_PROVIDER_EMAIL_ERROR_PCT
- API_PROVIDER_EMAIL_RATELIMIT_PCT
- API_PROVIDER_EMAIL_SEED

Fallback globals:
- API_PROVIDER_TIMEOUT_PCT
- API_PROVIDER_ERROR_PCT
- API_PROVIDER_RATELIMIT_PCT
- API_PROVIDER_SEED

## Run Tests

Use existing `bin/test.sh` plus new seeded tests to verify:
1. SMS and Email route to distinct providers (US1)
2. Breaker isolation (US2)
3. Deterministic outcomes under fixed seeds (US3)

### Provider Routing Verification (US1)

After starting the server:
1. POST an SMS to `/api/messages/sms` and an Email to `/api/messages/email`.
2. Wait ~200ms for the worker to process events.
3. GET `/metrics` and confirm:
	- `provider_sms_mms_attempts >= 1`
	- `provider_email_attempts >= 1`
4. Optionally inspect logs for `dispatch_attempt` and `dispatch_outcome` events showing `provider="sms-mms"` and `provider="email"`.
5. Confirm stored messages (`provider_name`) set by dumping the in-memory store via a temporary debug helper (to be added if needed) or extending conversations endpoint in future.

## Observability
- Logs include provider_name and outcome per dispatch.
- Metrics expose provider-labeled counters:
	- Attempts / Success / RateLimited / Error per provider
	- Breaker transitions per provider (`provider_sms_mms_breaker_transitions`, `provider_email_breaker_transitions`)
	- Global breaker transitions (`breaker_transitions`) retained
	- Routing failures (`invalid_routing`) if channel has no registered provider
	- Deterministic seed audit logs (`provider_seed` log events at startup)

## Deterministic Testing (US3)
Set seeds to produce reproducible sequences. Each provider maintains its own RNG stream.

Example:
```
export API_PROVIDER_SEED=777            # global fallback
export API_PROVIDER_SMS_ERROR_PCT=15
export API_PROVIDER_SMS_RATELIMIT_PCT=5
export API_PROVIDER_EMAIL_ERROR_PCT=20
export API_PROVIDER_EMAIL_RATELIMIT_PCT=10
```
Run server twice with identical env and send N SMS + N Email messages interleaved.
Metrics deltas (attempts, error, rate_limited) will match across runs.
For unit-level reproducibility, see test `deterministic.rs` which predicts counts from seed.

## Failure Simulation
Adjust pct variables to create error/timeouts for a single provider; confirm other provider unaffected and only that provider's breaker transitions.

Example (open SMS breaker quickly, leave Email healthy):
```
export API_BREAKER_ERROR_THRESHOLD=1
export API_PROVIDER_SMS_ERROR_PCT=100
export API_PROVIDER_EMAIL_ERROR_PCT=0
```
Send two SMS messages and one Email, then GET `/metrics`. Expect SMS breaker transition count ≥1, Email breaker transition count 0.
