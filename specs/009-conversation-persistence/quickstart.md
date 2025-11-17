# Quickstart: Conversation Persistence

## Goal
Ensure every message is consistently attached to a durable conversation with deterministic listing and safe snippets.

## Prerequisites
- PostgreSQL available (docker-compose.yml provided)
- Rust stable toolchain installed

## Steps
1. Apply migrations (includes conversations table and message FK).
2. Start the server.
3. Send an outbound message and an inbound reply between the same addresses on the same channel.
4. List conversations and verify a single conversation with message_count=2 and updated last_activity.
5. Retrieve messages for that conversation and verify snippets respect UTF-8 boundaries.

## Notes
- Plus-addressing for email is normalized to base address.
- Channels in scope: email and sms/mms.
- Snippet length defaults to 64 characters; configurable via service config.
