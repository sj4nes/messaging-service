# Implementation Plan: 003-poc-sqlx-migration

**Branch**: `003-poc-sqlx-migration` | **Date**: 2025-11-05 | **Spec**: specs/003-poc-sqlx-migration/spec.md
**Input**: Feature specification from `/specs/003-poc-sqlx-migration/spec.md`

**Note**: Plan covers Phase 0 (research) and Phase 1 (design/contracts) outputs to unblock implementation.

## Summary

PoC adding SQLx migrations and an admin utility (`db-migrate`) to manage PostgreSQL schema. Schema introduces normalized entities (customers, providers, endpoint_mappings, contacts, conversations, messages, attachments), dedup tables (`email_addresses`, `phone_numbers`, `email_bodies`, `xms_bodies`, `attachment_urls`) keyed by 64‑bit fast hashes, views for conversation overview/messages, UTC timestamps, and triggers for normalization + audit. A queue PoC (`inbound_events`) supports webhook decoupling.

## Technical Context

**Language/Version**: Rust (stable; MSRV stable−2)
**Primary Dependencies**: sqlx, sqlx_migrations, fasthash (for 64‑bit fast hashing)
**Storage**: PostgreSQL 15
**Testing**: cargo test; sqlx offline mode for schema checks; integration tests to apply migrations and assert constraints
**Target Platform**: Linux/macOS dev; containerized Postgres (docker-compose)
**Project Type**: Cargo workspace (existing) with `db-migrate` binary replacing `admin`
**Performance Goals**: Migrations apply in <5s locally; simple views return in <50ms on ~10k messages
**Constraints**: All timestamps stored as TIMESTAMPTZ (UTC); dedup acceptable collision risk (non‑cryptographic)
**Scale/Scope**: PoC scale (single instance, small datasets), preparing for growth

## Constitution Check

Security-First: No secrets in code/logs; credentials referenced indirectly. PII redaction in logs required. PASS
Test-First & Quality Gates: Plan includes migration application tests and constraint checks. PASS
Observability: Structured logs from tools minimal; DB audit via updated_at triggers. PASS for scope
Versioning & Change Control: JJ bookmarks; SemVer for crates; migrations versioned. PASS
Simplicity & SRP: Single utility `db-migrate`; focused schema. PASS

## Project Structure

### Documentation (this feature)

```text
specs/003-poc-sqlx-migration/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
```

### Source Code (repository root)

```text
crates/
├── core/                 # unchanged
├── server/               # unchanged
└── db-migrate/           # rename from admin; sqlx_migrations runner
```

**Structure Decision**: Rename `crates/admin` → `crates/db-migrate` to reflect purpose; add migrations/ directory consumed by sqlx.

## Phase 0: Outline & Research

- Hashing crate: Use `fasthash` (user preference) for 64‑bit fast hashes over normalized inputs. Alternatives: `ahash`, `fxhash`; fasthash adequate for dedup keys.
- ENUM strategy: Prefer constrained TEXT + CHECK in PoC to reduce migration friction; native ENUMs can follow later.
- Normalization: Emails lowercased; phone numbers to E.164 (use libphonenumber at app layer later; DB trigger keeps basic digit stripping as fallback). Bodies normalize whitespace + NFC.
- SQLx mode: Use `sqlx::migrate!` with migrations in `crates/db-migrate/migrations/`; support offline builds via SQLX_OFFLINE variable.

Output: research.md (decisions, rationale, alternatives)

## Phase 1: Design & Contracts

Artifacts to generate:
- data-model.md: Tables, key columns, FKs, UNIQUE constraints, views, triggers, indexes
- contracts/: No HTTP APIs added in this feature; include README stating N/A and reference data-model.md
- quickstart.md: How to run `db-migrate` locally against docker‑compose Postgres; create/apply migrations; rollback guidance
- Agent context update: run update-agent-context script for Copilot

## Complexity Tracking

N/A — no constitution violations anticipated.
