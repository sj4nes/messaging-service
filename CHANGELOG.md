## [Feature 008] Provider Routing, Breaker Isolation, Deterministic Seeds (2025-11-07)

Added:
- Channel → provider registry with mock `sms-mms` and `email` providers.
- Per-provider metrics (attempts, success, rate_limited, error) and breaker transition counters.
- Per-provider circuit breaker isolation; failures in one provider no longer impact others.
- Deterministic outcome sequencing via provider-specific RNG seeds with audit logs.
- Invalid routing metric to surface missing provider mappings.
- Unit & integration tests for routing, isolation, and deterministic reproducibility.

Improved:
- Expanded `Provider` trait documentation and internal message/result contracts.
- Quickstart guide now covers breaker isolation and deterministic testing procedures.

# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- TBD

## [0.2.0] - 2025-11-05

### Added
- Feature 004: Domain Events Catalog (transport-agnostic)
  - Canonical event envelope (JSON Schema Draft-07) with documented invariants and checklist
  - Event catalog covering Customers, Contacts, Providers, Channels, Conversations (purposes, required fields, invariants)
  - 21 validating example payloads under `specs/004-create-domain-events/contracts/events/examples/`
  - Validator script to check examples (`validate_examples.py`) and Make target `validate-events`
  - DX tasks: `dx-setup`, `py-venv`, `py-install-jsonschema`, `validate-events`, `rust-ensure`

### Changed
- Prerequisite script enhanced to infer active feature from JJ (bookmarks/log paths)

## [0.1.0] - 2025-11-06

### Added
- Feature 003: SQLx migrations PoC
  - Introduced `db-migrate` utility with `apply`, `new`, and `status` commands
  - Added initial schema: customers, providers, endpoint_mappings, contacts, conversations, conversation_participants, messages, attachments, message_attachments
  - Added dedup tables: email_bodies, xms_bodies, phone_numbers, email_addresses, attachment_urls (UNIQUE(hash))
  - Added indexes and audit/normalization triggers (`updated_at` maintenance; normalization stubs)
  - Added views: `conversation_overview`, `conversation_messages`
  - Added queue PoC: `inbound_events` + visibility/dispatch indexes
- Makefile tasks: migrate-apply, migrate-new, migrate-status, migrate-status-client
- Documentation updates: Quickstart, Data Model (sample inserts), README (SQLX_OFFLINE), DATABASE_URL in `.env.example`

### Changed
- Docker Compose maps Postgres to host port 55432 to avoid conflicts with local installations

