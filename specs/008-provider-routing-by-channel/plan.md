# Implementation Plan: Provider Routing By Channel

**Branch**: `008-provider-routing-by-channel` | **Date**: 2025-11-07 | **Spec**: ./spec.md  
**Input**: Feature specification from `/specs/008-provider-routing-by-channel/spec.md`

## Summary

Add explicit provider routing for outbound messages by channel (SMS, MMS, Email). Introduce a provider abstraction and registry keyed by channel, isolate circuit breaker state per provider, and record provider identity in logs/metrics and message records. No external API changes.

## Technical Context

**Language/Version**: Rust (repo targets 1.75+)  
**Primary Dependencies**: Axum (HTTP), Tokio (async), Serde (serde_json), internal middleware (rate limiting, circuit breaker), tracing  
**Storage**: PostgreSQL via SQLx (already wired for inbound events); in-memory store used for conversations/messages in current scope  
**Testing**: cargo test; HTTP scenarios via `bin/test.sh`; deterministic seeded tests for provider outcomes  
**Target Platform**: Linux/macOS server runtime  
**Project Type**: Single backend service (`crates/server`)  
**Performance Goals**: O(1) channel → provider lookup; no added latency in happy path  
**Constraints**: Preserve existing endpoints and response semantics; no secrets introduced; logs must remain redacted  
**Scale/Scope**: Same traffic as current tests; failure simulation only  

No open NEEDS CLARIFICATION items identified; defaults from spec apply.

## Constitution Check

- Security-First: No credentials added; inputs unchanged; rate limiting unaffected. PASS  
- Test-First and Quality Gates: Add unit/integration tests for routing, breaker isolation, seeded reproducibility. Target ≥80% coverage on new code. PASS (with added tests)  
- Observability: Add provider-labeled metrics and structured logs. PASS  
- Versioning/Change Control: Backwards compatible; no public contract changes. PASS  
- Simplicity/Single Responsibility: Minimal trait + registry; avoid over-engineering. PASS  

Re-check after Phase 1: Expected to remain PASS (no external changes introduced).

## Project Structure

### Documentation (this feature)

```text
specs/008-provider-routing-by-channel/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md (created by /speckit.tasks)
```

### Source Code (relevant paths)

```text
crates/server/src/
├── queue/outbound.rs          # Adjust routing and per-provider breaker usage
├── providers/                 # New Provider trait + implementations (sms_mms.rs, email.rs)
├── state/                     # AppState extended to hold provider registry and breakers
├── config.rs                  # Add per-provider config overrides
├── metrics.rs                 # Add per-provider counters/labels
└── store/messages.rs          # Tag outbound with provider_name (in-memory)
```

**Structure Decision**: Single backend service; add small, focused modules only; keep API surface stable.

## Phase 0 — Outline & Research

Unknowns: None. Research focuses on best practices for provider abstraction, per-provider breaker isolation, and label strategy for metrics.

Artifacts produced: `research.md` summarizing decisions and alternatives.

## Phase 1 — Design & Contracts

Outputs:
- `data-model.md`: OutboundMessage, Provider, ProviderRegistry, DispatchAttempt fields and relationships; validation notes.
- `contracts/README.md`: Notes that no external API contracts change in this feature; internal interface contract summarized.
- `quickstart.md`: How to configure per-provider overrides and observe routing, breakers, and metrics in logs/tests.
- Agent context update: run `.specify/scripts/bash/update-agent-context.sh copilot` to include new feature tech keywords.

Re-evaluate Constitution Check: PASS.

## Phase 2 — Tasks Outline (for /speckit.tasks)

1) Provider Abstraction & Registry
- Define Provider trait and OutboundMessage/DispatchResult types
- Implement SmsMmsMockProvider and EmailMockProvider
- Build provider registry (channel → provider)

2) Outbound Worker Routing & Breakers
- Parse channel from event payload
- Lookup provider and apply provider-specific breaker
- Record per-provider metrics and structured logs

3) Configuration & Metrics
- Per-provider overrides with fallback to global
- Add per-provider counters and labels

4) Persistence & Observability
- Tag outbound messages with provider_name in in-memory store
- Ensure logs include provider_name and outcomes

5) Tests
- Unit: registry mapping; breaker isolation; seeded determinism
- Integration: SMS vs Email routing; misconfig error path; metrics separation

6) Docs & Agent Context
- Update quickstart with env keys and verification steps
- Update agent context

## Complexity Tracking

No constitutional violations; no additional justification required.
