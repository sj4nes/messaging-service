# Quickstart: Running Migrations (PoC)

This feature introduces SQLx migrations and a `db-migrate` utility (renamed from `admin`). Until the rename lands, commands may use the existing `admin` crate name.

## Prerequisites

- Docker and Docker Compose available
- Rust toolchain installed

## Start Postgres (dev)

- Use the provided docker-compose.yml to start PostgreSQL locally.
- Ensure the database URL is available to the tool (e.g., `DATABASE_URL=postgres://...`). For local dev, use `.env` with the same precedence rules as the server.

## Apply migrations

- Ensure DATABASE_URL is set, then apply migrations via the utility:

```sh
# apply all pending migrations (compose maps host 55432 -> container 5432)
DATABASE_URL=postgres://messaging_user:messaging_password@localhost:55432/messaging_service \
  cargo run -p db-migrate -- apply
```

- Expected behavior:
  - On success: exit 0, prints applied migrations
  - On error: non-zero exit, clear diagnostics

## Create a new migration

- Use the utility to create a new timestamped migration file with `-- migrate:up` and `-- migrate:down` sections:

```sh
# create a new migration file like 20251105123456_add_customers.sql
cargo run -p db-migrate -- new add_customers
```

- Write SQL for schema changes; prefer additive changes in PoC.

## Notes

- SQLX_OFFLINE can be used to skip DB connectivity checks in CI; local dev should run online.
- All timestamps should be stored in UTC; triggers will help enforce this.
- Dedup tables rely on application-provided normalized input and 64-bit hash values.
