# Audit Working Notes

- Seed function `SeedMinimumIfNeeded` is guarded by atomic flag; invoked on each list call before DB query.
- Fallback enabled via `INMEMORY_FALLBACK`; list methods normalize nil slices to empty arrays.
- Metrics parity: worker_processed increases only when the inbound DB worker processes events. If no DATABASE_URL is configured (local runs), inbound worker is disabled; counter remains 0 by design (don’t simulate).
- Potential improvement: eager seed at startup vs lazy on first list.
- Legacy conversation ID mapping logic only applied for ID "1".
