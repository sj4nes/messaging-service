# Quickstart: Wire PostgreSQL Store

Created: 2025-11-06

This feature wires inbound_events persistence and a background worker using PostgreSQL.

## Prerequisites

- PostgreSQL running (docker-compose up -d)
- DATABASE_URL exported or present in .env
- Apply migrations:

```bash
make migrate-apply
```

## Run the server

```bash
make run-server
```

## Exercise inbound events and processing

1. Send inbound webhook examples (see prior quickstart in 006-unified-messaging).
2. Verify events are persisted (psql):

```bash
psql "$DATABASE_URL" -c "SELECT id, channel, status, attempt_count FROM inbound_events ORDER BY created_at DESC LIMIT 10;"
```

3. Verify messages are persisted:

```bash
psql "$DATABASE_URL" -c "SELECT id, channel, conversation_key, timestamp FROM messages ORDER BY timestamp DESC LIMIT 10;"
```

4. Verify conversations list API reflects DB state:

```bash
curl -sS http://localhost:8080/api/conversations | jq
```

## Configuration knobs

- BATCH_SIZE (env): number of events claimed per cycle (default 10)
- CLAIM_TIMEOUT_SECS (env): release claims after timeout (default 60)
- MAX_RETRIES (env): processing retries before dead_letter (default 5)
- BACKOFF_BASE_MS (env): base backoff (default 500)

## Troubleshooting

- If migrations fail, ensure SQLX_OFFLINE is unset for local dev or update offline data accordingly.
- Check logs for mock provider and worker events; use mock=true filter in traces.
