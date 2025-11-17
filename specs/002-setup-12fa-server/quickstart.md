# Quickstart — Setup 12‑Factor Server Bootstrap

This feature bootstraps a minimal service with configuration via environment/.env, a health endpoint, and structured logging. The repository will organize code as a Cargo workspace with a shared library, a server binary, and an admin binary for migrations (stub).

## Prerequisites

- Rust (latest stable)
- PostgreSQL available if testing admin migration later (not required for health demo)
- Make (optional), Bash

## Configuration

- Environment variables:
  - `PORT` (default: 8080)
  - `HEALTH_PATH` (default: /healthz)
  - `LOG_LEVEL` (default: info)
- .env file (optional): values are loaded but overridden by environment variables.

## Try it

```bash
# From repository root
# Run server (will be crates/server in the workspace)
cargo run -p messaging-server

# Health check
curl -i http://localhost:8080/healthz
```

## Notes

- Logs should indicate configuration source and resolved values at startup.
- Health route must be accessible without authentication.
- Admin migration binary will be a stub initially (no-op), to be wired with sqlx_migrations later.