# Contracts for 004-create-domain-events

This feature does not introduce HTTP APIs. Instead, it defines a transport‑agnostic event catalog with a common envelope and example payloads. See `events/`:

- envelope.schema.json — JSON Schema for the common event envelope
- catalog.md — Human-readable list of event types, required fields, and semantics
- examples/*.example.json — Sample event instances validated against the envelope schema
