# Remediation Tasks Mapping

Source gaps: `gap-inventory.md` / `gap-inventory.json`
Generated: 2025-11-13

| Task ID | Gap ID | Title | Priority | Estimate | Dependencies | Acceptance Criteria | Notes |
|---------|--------|-------|----------|----------|--------------|---------------------|-------|
| TASK-001 | GAP-001 | Document simulated worker_processed increments and add TODO for real worker | P2 | 1h | - | Gap inventory & metrics-parity include explicit note; status Closed | Simulation currently in handlers; clarify provisional nature |
| TASK-002 | GAP-002 | Decide and apply seeding strategy (eager or documented lazy) | P3 | 2h | TASK-001 (optional) | README + research updated; seed timing stable; first list call deterministic | If eager chosen, ensure idempotency guard prevents duplicates |
| TASK-003 | GAP-003 | Provide JSON metrics shim endpoint mirroring snapshot | P3 | 3h | TASK-001 | /metrics.json returns JSON fields listed in metrics-parity.md; documented in quickstart | Avoid breaking existing Prometheus text endpoint |
| TASK-004 | GAP-004 | Add provider breaker transition increments | P3 | 3h | - | provider_*_breaker_transitions counters increase after forced error + recovery scenario | Include test harness snippet or manual steps |
| TASK-005 | GAP-005 | Honor pageSize query parameter & align default semantics | P2 | 5h | - | Responses clamp pageSize to 50; pageSize<=0 defaults to 50; docs updated | Add regression tests asserting meta.page_size reflects clamped value |
| TASK-006 | GAP-006 | (Closed) Verification regression test for legacy id=1 fallback | P4 | 1h | - | Test proves fallback works after DB seed (non-critical) | Optional hardening task |

## Dependency Graph
- TASK-001 precedes TASK-003 (shim depends on chosen metrics representation).
- TASK-002 independent (may reference TASK-001 decision).
- TASK-005 independent; can run in parallel with others.
- TASK-004 independent.

## Parallelization
- TASK-002, TASK-004, TASK-005 can proceed concurrently once TASK-001 decision locked.

## Progress Dashboard (Updated)
| Metric | Count |
|--------|-------|
| Total Tasks | 6 |
| Open | 3 |
| Closed | 3 |
| In Progress | 0 |
| Blocked | 0 |
| Completion % | 50% |

Completion % = Closed / Total.

## Next Steps
1. Confirm strategy for GAP-001 (implement vs document). Set TASK-001 status accordingly.
2. (Done) TASK-005 executed — pagination aligned to new policy (cap 50, 0→50); tests added and passing.
3. Introduce `/metrics.json` (TASK-003) if JSON parity required by external tooling.
4. Update progress dashboard after each completion.

## Acceptance Criteria Details
- Metrics JSON shim: must include all fields listed in `metrics-parity.md` with identical naming; missing fields default to 0.
- Pagination: pageSize>0 must constrain returned items length ≤ pageSize; meta.page_size equals requested pageSize; page_size=0 retains current unbounded behavior.
- Provider breaker transitions: create failure injection to force breaker open & close; counters increment at least once; documented reproduction.

## Risk Notes
- Implementing inbound worker (TASK-001) may introduce concurrency concerns; keep scope minimal (process queued events only).
- JSON shim duplication risk; ensure single metrics snapshot source feeding both formats to avoid drift.

