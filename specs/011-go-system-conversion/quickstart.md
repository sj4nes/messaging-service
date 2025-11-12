# Quickstart: Go Messaging Service (Feature 011)

Date: 2025-11-12

This guide shows how to build, run, and verify the Go implementation side-by-side with the existing Rust service.

## Prerequisites
- Go 1.22+
- Docker (optional for Compose)
- macOS/Linux shell tools: curl, jq (optional)

## Build & Run (local)

1) Build

```
make go.build
```

2) Run (defaults: PORT=8080, /healthz, /metrics)

```
make go.run
```

3) Health check

```
curl -i http://localhost:8080/healthz
```

4) Metrics

```
curl -s http://localhost:8080/metrics | head -n 20
```

## API Smoke

Send SMS (202 Accepted expected):

```
curl -i \
  -H 'Content-Type: application/json' \
  -d '{"from":"+12016661234","to":"+18045551234","type":"sms","body":"Hello","attachments":null,"timestamp":"2024-11-01T14:00:00Z"}' \
  http://localhost:8080/api/messages/sms
```

List conversations (200 OK expected):

```
curl -s -H 'Accept: application/json' http://localhost:8080/api/conversations | jq .
```

## Auth (optional)

Auth is disabled by default. To enable bearer-token auth for protected routes (/api/...):

```
export AUTH_ENABLED=true
export AUTH_TOKENS=devtoken1,devtoken2
PORT=8080 make go.run
```

Use the token in Authorization header:

```
curl -i \
  -H 'Authorization: Bearer devtoken1' \
  -H 'Content-Type: application/json' \
  -d '{"from":"+12016661234","to":"+18045551234","type":"sms","body":"Hello","attachments":null,"timestamp":"2024-11-01T14:00:00Z"}' \
  http://localhost:8080/api/messages/sms
```

## Rate limits

- Public limiter: RATE_LIMIT_PUBLIC_RPS (default 5), RATE_LIMIT_PUBLIC_BURST (10)
- Protected limiter: RATE_LIMIT_PROTECTED_RPS (default 2), RATE_LIMIT_PROTECTED_BURST (5)

Example:

```
export RATE_LIMIT_PUBLIC_RPS=10
export RATE_LIMIT_PUBLIC_BURST=20
```

## Logging & Redaction

- LOG_LEVEL: debug|info|warn|error (default info)
- LOG_REDACT=true enables masking of sensitive values in log messages and string fields.
- LOG_REDACT_PATTERNS: comma-separated regex list (defaults include Authorization/API key/token patterns).

Example:

```
export LOG_LEVEL=debug
export LOG_REDACT=true
export LOG_REDACT_PATTERNS='(?i)authorization: .*,(?i)token=\w+'
PORT=8080 make go.run
```

## SSRF Protection (Outbound)

The internal HTTP client validates destinations against an allowlist and blocks private/link-local ranges.

- SSRF_ALLOWLIST: comma-separated host suffixes; default: example.com

## pprof (optional)

Enable pprof endpoints for profiling:

```
export PPROF_ENABLED=true
export PPROF_PATH=/debug/pprof
PORT=8080 make go.run
```

Browse:
- http://localhost:8080/debug/pprof/
- http://localhost:8080/debug/pprof/profile

## Docker Compose

Run both services (Go + Rust + Postgres):

```
make docker-up
```

Check status:

```
docker-compose ps
```

Logs:

```
make docker-logs
```

## Test Suite

A contract-style HTTP runner is available:

```
./bin/test.sh
```

- Use FORCE_RESTART=true to force a fresh server:

```
FORCE_RESTART=true ./bin/test.sh
```

## Troubleshooting

- Address in use: set PORT to a free port (`PORT=8081 make go.run`).
- Missing jq: install via Homebrew (`brew install jq`) or skip JSON formatting in tests.
- Auth 401: ensure `AUTH_ENABLED=true` and `Authorization: Bearer <token>` header present.
