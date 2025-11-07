# Research: Provider Routing By Channel

Date: 2025-11-07

## Decisions

- Provider Abstraction: Introduce a simple trait (name, dispatch). Rationale: isolates routing logic, enables test doubles, maintains simplicity.
- Registry Mapping: Channel → Provider (sms, mms → sms-mms provider; email → email provider). Rationale: O(1) lookup and clear ownership boundaries.
- Per-Provider Circuit Breaker: One breaker per provider instance; 429 does not trip breaker. Rationale: isolates failures and prevents cross-channel impact.
- Configuration: Per-provider override envs with fallback to existing global keys. Rationale: preserves backwards compatibility.
- Metrics & Logs: Provider-labeled counters; structured logs include provider and outcome. Rationale: observability and auditability.
- Persistence Tagging: Add provider_name to in-memory outbound records for correlation. Rationale: troubleshooting and future retries.

## Alternatives Considered

- Single Global Provider with Internal Switch: Rejected due to poor isolation and extensibility.
- Dynamic Plugin System for Providers: Rejected as over-engineering for current scope; increases complexity without external integrations.
- Per-Customer Routing Rules: Deferred; not required for this step and increases scope.

## Clarifications

No open NEEDS CLARIFICATION items; defaults from spec used.
