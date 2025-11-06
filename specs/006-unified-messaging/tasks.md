# Tasks: Unified Messaging (Feature 006)

Legend: [ ] todo, [x] done

## Phase 1 — Contracts & Docs
- [x] T600: Author OpenAPI contracts for outbound send (SMS/Email)
- [x] T601: Add inbound injection endpoint + mock runtime config endpoints to OpenAPI
- [x] T602: Write Quickstart with curl examples, metrics, config

## Phase 2 — Implementation
- [ ] T610: Define request DTOs and validators for SmsRequest, EmailRequest
- [ ] T611: Implement POST /api/messages/sms and /api/messages/email to enqueue outbound messages
- [ ] T612: Implement in-process provider mock with configurable probabilities (timeout/error/429) and deterministic seed
- [ ] T613: Implement GET/PUT /api/provider/mock/config to read/update mock config (in-memory)
- [ ] T614: Implement POST /api/provider/mock/inbound to accept and normalize inbound events
- [ ] T615: Add worker/queue consumer to dispatch outbound messages via provider mock with rate limiter + circuit breaker integration
- [ ] T616: Add metrics for dispatch attempts/outcomes and breaker transitions; expose via /metrics
- [ ] T617: Wire configuration from env vars (API_PROVIDER_*), with file overrides, and sane defaults
- [ ] T618: Structured logging for dispatch and inbound paths; include correlation IDs

## Phase 3 — Stretch: Conversations
- [ ] T620: Implement in-memory conversation grouping (channel, normalized_from, normalized_to)
- [ ] T621: Add GET /api/conversations and GET /api/conversations/{id}/messages (in-memory)

## Phase 4 — Tests & Validation
- [ ] T630: Unit tests for DTO validation and normalization edge cases
- [ ] T631: Scenario tests: success, 429 with Retry-After, 5xx causing breaker open/half-open/close
- [ ] T632: Deterministic seed tests for reproducibility
- [ ] T633: Contract tests against OpenAPI via JSON cases (bin/test.sh)

## Phase 5 — DX Polish
- [ ] T640: README updates linking to Quickstart and contracts
- [ ] T641: Example env file and Make targets for running mocks and scenario tests

Notes:
- Keep persistence in-memory for this feature unless otherwise required; ensure seams allow swapping to SQLx later.
- Favor small, composable modules for provider mock and queue consumer.
