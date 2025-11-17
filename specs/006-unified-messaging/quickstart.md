# Unified Messaging Quickstart (Feature 006)

## Goal
Rapidly exercise outbound and inbound messaging flows with provider mock failure simulation.

## Prerequisites
- Rust toolchain (stable)
- `make` and `docker` (if adding Postgres later; not required now)
- Set any API_* env overrides as needed.

## Start Server
```
make run  # or cargo run -p server
```

## Outbound Send Examples
Send SMS:
```
curl -X POST localhost:3000/api/messages/sms \
  -H 'Content-Type: application/json' \
  -d '{"from":"+15550001","to":"+15550002","type":"sms","body":"Hello","timestamp":"2025-01-01T00:00:00Z"}'
```
Send Email:
```
curl -X POST localhost:3000/api/messages/email \
  -H 'Content-Type: application/json' \
  -d '{"from":"alice@example.com","to":"bob@example.com","body":"Hello Bob","timestamp":"2025-01-01T00:00:00Z"}'
```
Response will be 202 Accepted with `{ "status": "accepted" }` if enqueued.

## Inbound Injection (Mock)
Inject inbound events using the mock provider endpoint:
```
curl -X POST localhost:3000/api/provider/mock/inbound \
  -H 'Content-Type: application/json' \
  -d '{
        "channel":"sms",
        "from":"+15550002",
        "to":"+15550001",
        "type":"sms",
        "body":"Reply: got it",
        "timestamp":"2025-01-01T00:01:00Z"
      }'
```
For email inbound:
```
curl -X POST localhost:3000/api/provider/mock/inbound \
  -H 'Content-Type: application/json' \
  -d '{
        "channel":"email",
        "from":"bob@example.com",
        "to":"alice@example.com",
        "body":"Re: Hello",
        "timestamp":"2025-01-01T00:01:00Z"
      }'
```

## Failure Simulation
Configure probabilities via env vars (planned) or at runtime (mock config endpoints):
```
# Percent (0-100) chance of simulated provider timeout
export API_PROVIDER_TIMEOUT_PCT=5
# Percent chance of provider 5xx error
export API_PROVIDER_ERROR_PCT=2
# Percent chance of provider rate-limit (429)
export API_PROVIDER_RATELIMIT_PCT=1
# Seed for deterministic sequence
export API_PROVIDER_SEED=12345
```
Restart the server after setting envs.

Or configure at runtime (in-memory):
```
curl localhost:3000/api/provider/mock/config

curl -X PUT localhost:3000/api/provider/mock/config \
  -H 'Content-Type: application/json' \
  -d '{"timeout_pct":10,"error_pct":2,"ratelimit_pct":1,"seed":42}'
```

## Metrics
```
curl localhost:3000/metrics
```
Expect counters for rate-limited and breaker-open events.

## Conversations (Stretch)
List conversations:
```
curl localhost:3000/api/conversations?page=1&pageSize=20
```
List messages in a conversation:
```
curl localhost:3000/api/conversations/{id}/messages?page=1&pageSize=50
```

## Troubleshooting
- 400 Bad Request: Validate body matches OpenAPI schema.
- 429 Too Many Requests: Backoff using `Retry-After` header.
- 415 Unsupported Media Type: Ensure `Content-Type: application/json`.
- Deterministic failures: Set `API_PROVIDER_SEED` for reproducibility in tests.

## Next
Implement inbound endpoint + provider mock layer; wire circuit breaker metrics; persist (optional) conversation grouping.
