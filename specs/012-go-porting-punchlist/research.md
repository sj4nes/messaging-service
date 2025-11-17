# Research: Go Porting Punchlist (Phase 0)

## Decisions Summary

| Decision ID | Topic | Decision | Rationale | Alternatives Considered |
|-------------|-------|----------|-----------|-------------------------|
| D-001 | List Response Normalization | Use empty JSON arrays `[]` rather than `null` for zero items | Aligns with existing contract tests and common REST conventions; improves client simplicity | Preserve `null` (breaks tests), wrapper object with explicit count field only |
| D-002 | Metrics Parity | Expose existing counters; add documentation clarifying worker_processed simulation; defer adding real worker metrics until worker implemented | Avoids misleading synthetic increments; keeps observability honest | Stub worker goroutine (adds complexity), remove counter entirely (loses parity signal) |
| D-003 | Seeding Strategy | Keep deterministic seed; in current reference, seeding is lazy (guarded by atomic) and invoked before first list/db operations; document behavior | Provides stable baseline for tests without extra startup latency; avoids duplicate inserts | Eager seed at startup (slightly slower boot), dynamic test-time seeding (adds harness complexity), disabling seed (empty responses failing tests) |
| D-004 | Background Worker | Defer full queue/worker implementation; document absence and recommend incremental adoption (phase after parity closure) | Reduces scope; prevents rushed concurrent code; parity can be achieved without asynchronous processing | Implement full worker now (scope creep), partial polling hack (fragile) |
| D-005 | Gap Inventory Format | Use structured markdown + optional JSON export for automated tooling | Human readable & machine parsable; easy diffing in PRs | Plain text list (harder to parse), only JSON (less readable) |
| D-006 | Task Mapping | One remediation task per gap (merge trivial gaps) | Clear traceability | Large umbrella tasks (loss of granularity), per-line tasks (too granular) |

## Detailed Rationale

### D-001 Empty vs Null Arrays
Null collection responses complicate client code (nil checks). Tests expect arrays; standard REST practice favors empty arrays. No compatibility issues anticipated.

### D-002 Metrics Parity
True worker metrics require asynchronous processing. Simulated increments inside handlers risk misrepresenting real throughput. Decision: document current state; keep counter but only increment where true processing is analogous or mark clearly as provisional.

### D-003 Seeding
Deterministic seed ensures reproducible listing tests. The current implementation uses a one-time guard and performs seed lazily (before first list/db usage) to minimize startup costs. Alternative dynamic seeding per test would require harness changes. Risk of stale seed considered acceptable.

### D-004 Worker Deferral
Implementing queue, retry, backoff, error handling expands scope significantly. Documentation plus remediation task scheduling provides transparency without premature complexity.

### D-005 Gap Inventory Format
Markdown table with YAML front-matter (optional) enables human review; JSON export supports future automation. Hybrid approach chosen.

### D-006 Task Granularity
Mapping each gap directly maintains clarity in progress tracking. Micro-gaps can be merged when they share acceptance criteria.

## Alternatives Rejected (Concise)
- Full asynchronous worker now: scope creep; testing complexity.
- Remove worker_processed metric: loses important parity indicator.
- Keep null list items: fails tests; poorer client ergonomics.
- Dynamic per-test seeding: unnecessary harness changes.

## Open Items
No unresolved clarifications; all spec unknowns resolved.

## Next Steps
Proceed to Phase 1 design: data-model.md, contracts schemas, quickstart guide, agent context update.
