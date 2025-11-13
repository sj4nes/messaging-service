# Quickstart: Go Porting Punchlist Parity Audit

## Purpose
Guide contributors to generate, review, and validate the parity gap inventory between Rust reference and Go port, and confirm closure.

## Prerequisites
- Repository cloned
- Go and Rust toolchains installed
- PostgreSQL via docker-compose (optional for seed verification)

## Steps

### 1. Generate/Update Gap Inventory (Manual)
1. Review test failures / differences.
2. Add or update `gap-inventory.md` (or JSON from schema) under this feature directory (future automation TBD).
3. Ensure each gap has unique `GAP-###` id and acceptance criteria.

### 2. Run Contract Tests
```
./bin/test.sh
```
Confirm 0 failures after remediation; if list endpoints still fail, verify empty arrays serialized correctly.

### 3. Verify Metrics Parity
Curl metrics endpoint (Go vs Rust) and confirm presence of counters (startup, worker_processed or documented absence).

Example (macOS/Linux, requires jq):

```
BASE_URL=${BASE_URL:-http://localhost:8080}
curl -sS -H 'Accept: application/json' "$BASE_URL/metrics" | jq '{
	ts_unix_ms,
	rate_limited,
	breaker_open,
	dispatch_attempts,
	dispatch_success,
	provider_sms_mms_attempts,
	provider_email_attempts,
	worker_processed
}'
```

Notes:
- If `worker_processed` remains 0 locally, that likely means the inbound DB worker is disabled (no DATABASE_URL). This is expected; the test runner treats the wait as best-effort.
- If the Go port exposes Prometheus text instead of JSON, use `curl .../metrics | grep` to spot counters or provide a JSON shim for the audit.

### 4. Generate Closure Report
Create JSON matching `parity-report.schema.json` with counts; criticalRemaining must be 0.

### 5. Review & Sign Off
Share report in PR; obtain approvals referencing Constitution gates.

## Common Issues
| Symptom | Cause | Fix |
|---------|-------|-----|
| Null items in list responses | Slice not initialized | Initialize to empty slice before marshal |
| Missing worker_processed metric | No worker increments | Document absence; remove test dependency or implement stub |
| Seed data absent | Seed not invoked early | Call seed helper during store init |

## Next Steps
Proceed to `/speckit.tasks` to create actionable remediation tasks.
