# Quickstart: Domain Events Catalog (Feature 004)

This feature delivers a transport‑agnostic event catalog, a common event envelope JSON Schema, and example payloads.

## Contents

- contracts/events/envelope.schema.json — JSON Schema for the common envelope
- contracts/events/catalog.md — Event names, purposes, required fields
- contracts/events/examples/*.example.json — Example payloads
- data-model.md — Entities, envelope, event families
- research.md — Decisions, rationale, alternatives

## Validate examples

Use any JSON Schema validator. Here are two common options:

- Node.js (AJV CLI):
  - Install: `npm -g install ajv-cli`
  - Validate: `ajv validate -s contracts/events/envelope.schema.json -d contracts/events/examples/*.example.json`

- Python (jsonschema):
  - Install: `pip install jsonschema`
  - Validate (one file): `python -c "import json,sys; from jsonschema import validate, Draft7Validator as V; import pathlib; s=json.load(open('contracts/events/envelope.schema.json')); V.check_schema(s); d=json.load(open('contracts/events/examples/customer_created.example.json')); validate(d, s); print('ok')"`

## Extending the catalog

- Add new events to `contracts/events/catalog.md` and provide an example payload in `contracts/events/examples/`
- Keep envelope fields intact; bump `version` for backward-compatible additions; introduce a new `event_name` for breaking changes
- Avoid PII; prefer stable identifiers; redact sensitive data before adding examples

## Conversation identity

- Conversations are identified for routing/grouping by (customer_id, channel_id, contact_id) per the spec
- Conversation lifecycle events: Created, Updated, Closed, Reopened, ParticipantAdded/Removed, Archived
