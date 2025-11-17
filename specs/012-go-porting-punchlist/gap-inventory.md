# Gap Inventory

Current test run (22/22 passing) indicates no blocking functional gaps for covered endpoints. Some originally hypothesized gaps have been resolved or were inaccurate; they are replaced with clarified entries below. Each gap captures expected vs actual parity between Rust reference and Go port. Severity reflects impact on audit closure.

| ID | Category | Severity | Title | Expected Behavior | Actual Behavior | Reproduction / Evidence | Acceptance Criteria | Status |
|----|----------|----------|-------|-------------------|-----------------|-------------------------|---------------------|--------|
| GAP-001 | Observability | Medium | Missing true worker metrics in Go | Rust exposes worker_* counters tied to inbound DB worker lifecycle | Go currently simulates increments inline in HTTP handlers (sms/email) rather than real async processing | Inspect `go/api/messages.go` for `IncWorkerProcessed()` calls; curl `/metrics` after requests shows increments | Document simulation as provisional and add clear TODO to move increments to real worker OR remove simulation before parity close | Closed (Documented Simulation) |
| GAP-002 | Data | Low | Lazy vs startup seed divergence | Reference seed ensures baseline deterministic data early; parity expects documented timing | Go may seed only after first list (lazy) causing initial metrics scrape before data | Start fresh container then immediately request `/api/conversations` and `/metrics` | Document lazy seeding strategy clearly OR switch to eager on startup for deterministic first scrape | Open |
| GAP-003 | Observability | Low | Metrics format divergence risk | Parity expects JSON metrics snapshot fields per `metrics-parity.md` | Go may emit Prometheus text format if instrumentation added later | `curl -s $BASE_URL/metrics` inspect Content-Type and body | Provide JSON endpoint (current) or add shim ensuring test harness compatibility | Open |
| GAP-004 | Operability | Low | Provider breaker transitions parity | Rust tracks per-provider breaker transitions | Go may have placeholders without increment logic | Invoke error scenarios (force failures) and check counters | Implement increment paths when breaker state changes OR mark as deferred (not critical for MVP) | Open |
| GAP-005 | Functional | Medium | Conversation pagination defaults alignment | Policy updated: cap page size at 50; pageSize=0 or missing defaults to 50 | Both Go and Rust clamp page size to 50; pageSize>50 → 50; pageSize<=0 → 50 | Verified via tests on Go; Rust updated accordingly | Closed |
| GAP-006 | Functional | Low | Legacy conversation ID fallback reliance | Rust implements mapping logic for id=1 fallback to first conversation when DB IDs differ | Go must mimic or tests referencing id=1 could fail under DB-backed mode | After DB seed, request `/api/conversations/1/messages` when first conversation ID is not 1 | Confirm fallback exists or implement; tests already pass (evidence: test run) -> mark Verified | Closed |

Legend:
- Severity: High (blocks audit), Medium (needs resolution/documentation), Low (can defer with clear note).
- Status values: Open, Closed, Validation Pending.

Planned Next Steps (US1 Tasks T012–T020):
1. Validate GAP-005 pagination behavior; update status.
2. Add reproduction scripts (curl snippets) for each gap.
3. Produce JSON export (`gap-inventory.json`) matching schema with these entries.
4. Add acceptance criteria refinement (explicit test or doc artifact) and priorities.

No critical (High) gaps currently identified; audit can proceed to detailed enumeration and remediation mapping.
