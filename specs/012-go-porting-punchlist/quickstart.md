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
