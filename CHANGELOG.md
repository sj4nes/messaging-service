# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- TBD

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

