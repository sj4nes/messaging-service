# Gap Inventory (Draft)

| ID | Category | Priority | Expected Behavior | Actual Behavior | Reproduction | Acceptance Criteria | Status |
|----|----------|----------|-------------------|-----------------|-------------|---------------------|--------|
| GAP-001 | Functional | High | List endpoints return empty arrays when no data | Messages endpoint returns null items (if any) | Run bin/test.sh list cases | Items array serializes as [] consistently | Open |
| GAP-002 | Observability | Medium | worker_processed metric reflects actual async processing | Counter increment simulation inline handlers | Inspect metrics endpoint after sends | Metric documented as provisional or true worker implemented | Open |
| GAP-003 | Data | Low | Seed deterministic baseline aligns with reference dataset | Seed executes only after first list, may not populate before metrics scrape | Start server; immediately curl list endpoints | Seed invoked at startup or documented timing | Open |

> Extend with additional gaps during T012–T015 tasks.
