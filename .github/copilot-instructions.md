# messaging-service Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-11-05

## Active Technologies
- PostgreSQL via SQLx + sqlx_migrations (admin stub only in this feature) (002-setup-12fa-server)
- Rust (repo), Markdown + JSON Schema (this feature) + python-jsonschema (local validator), optional AJV for consumers (004-create-domain-events)
- Rust 1.75+ (repo currently builds on stable) + Axum (HTTP), Tokio (async runtime), Serde, Tracing; internal modules for rate limiting, circuit breaker, idempotency, inbound queue (006-unified-messaging)
- PostgreSQL via SQLx planned; for this feature, in-memory acceptable if persistence not yet wired for messages; conversation stretch may use in-memory grouping (006-unified-messaging)
- Rust (stable; repo targets 1.75+) + Axum, Tokio, SQLx, Serde, Tracing (007-wire-postgresql-store)
- PostgreSQL (docker-compose; SQLx migrations) (007-wire-postgresql-store)
- Rust 1.75 (stable toolchain) + Axum (HTTP), Tokio (async), SQLx (PostgreSQL), Serde (serialization), Tracing (observability) (009-conversation-persistence)
- PostgreSQL (durable conversations/messages); in-memory only for fallback when DB unavailable (009-conversation-persistence)

- Bash (POSIX-compatible) + Jujutsu CLI (`jj`), Git CLI (`git`) (001-jujutsu-scm-support)

## Project Structure

```text
src/
tests/
```

## Commands

# Add commands for Bash (POSIX-compatible)

## Code Style

Bash (POSIX-compatible): Follow standard conventions

## Recent Changes
- 009-conversation-persistence: Added Rust 1.75 (stable toolchain) + Axum (HTTP), Tokio (async), SQLx (PostgreSQL), Serde (serialization), Tracing (observability)
- 008-provider-routing-by-channel: Added [if applicable, e.g., PostgreSQL, CoreData, files or N/A]
- 007-wire-postgresql-store: Added Rust (stable; repo targets 1.75+) + Axum, Tokio, SQLx, Serde, Tracing


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
