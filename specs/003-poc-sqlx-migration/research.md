# Research: SQLx Migrations, Hashing, Normalization

Date: 2025-11-05
Feature: 003-poc-sqlx-migration

## Decisions

- Hashing: Use `fasthash` 64-bit (fast, non-cryptographic) for dedup keys across emails, phones, bodies, attachment URLs.
  - Inputs are normalized first per type (email lowercased; phone digits/E.164; bodies trimmed + whitespace normalized + NFC).
  - Collision risk accepted for PoC; UNIQUE(hash) enforced.
- ENUM Strategy: Start with TEXT + CHECK constraints for kinds/statuses in this PoC to simplify migrations; revisit native enums later.
- SQLx Migrations: Use `sqlx::migrate!()` with migration files under `crates/db-migrate/migrations/`.
  - Support SQLX_OFFLINE for CI; keep a dev Postgres via docker-compose.
- Timestamps: Store all as TIMESTAMPTZ (UTC). Convert inputs to UTC. `updated_at` maintained by triggers.
- Triggers: BEFORE INSERT on dedup tables to compute normalized value + hash.
  - `email_bodies` and `xms_bodies` have analogous normalization.
  - BEFORE INSERT/UPDATE triggers maintain `updated_at` on core tables.

## Alternatives Considered

- Hashing: `ahash`, `fxhash`, or cryptographic hashes (SHA-256).
  - Chosen `fasthash` for speed and sufficient collision profile; SHA‑256 overkill and larger.
- ENUMs: PostgreSQL native ENUMs.
  - Chosen TEXT + CHECK for agility; native ENUM migrations are heavier.
- DB-level normalization: Full E.164 with an extension.
  - Chosen minimal DB trigger normalization; full canonicalization at application layer later.

## Open Questions (Deferred)

- Cross-table `messages.body_id` referential safety to the correct dedup table based on message kind — handled in application code for PoC; could model with separate columns or polymorphic FKs later.
- Partitioning strategy for large tables (messages, inbound_events) — out of scope for PoC.
