# Research: Conversation Persistence & Unification

## Decisions

1. Conversation Key Normalization
- Decision: Key is (channel, ordered normalized participants). Email lowercased with plus-tag equivalence; phone digits with optional leading '+', strip spaces/dashes, keep country code if provided.
- Rationale: Ensures direction-agnostic, deterministic mapping while preserving user intent and internationalization basics.
- Alternatives: Treat email plus-tags as distinct (more fragmentation); full E.164 validation (more complexity, defers to later feature).

2. Upsert Strategy
- Decision: Application-level transaction with INSERT ... ON CONFLICT (channel, participant_a, participant_b) DO UPDATE RETURNING id; update message_count and last_activity atomically with message insert.
- Rationale: Avoid trigger divergence; maintain single source of truth in application code; resilient under concurrency.
- Alternatives: DB triggers for counters (harder to test/version); separate read-before-write (race prone).

3. Idempotency & Duplicate Handling
- Decision: Use existing message idempotency keys; on duplicate message insert, do not increment conversation counts.
- Rationale: Prevents inflated statistics; aligns with reliability goals.
- Alternatives: Post-aggregation reconciliation (delayed correction; adds complexity).

4. Deterministic Pagination
- Decision: Order conversations by last_activity_at DESC, tiebreak by id DESC; messages ordered by effective timestamp.
- Rationale: Stable ordering for paging; matches user expectations for "recent first".
- Alternatives: Primary-key only ordering (less user-friendly); secondary sort by participant (inconsistent with recency focus).

5. UTF-8 Safe Snippets
- Decision: Snippet length configurable (default 64 chars); truncate on character boundary using Unicode-aware APIs.
- Rationale: Avoid corrupting text and maintain readability, especially for emoji/non-Latin scripts.
- Alternatives: Byte-length truncation (risks broken code points); strip emojis (user-hostile).

6. Backfill Process
- Decision: Batch process messages with NULL conversation_id; compute key, upsert conversation, update FK; finish with aggregate recompute of message_count and last_activity.
- Rationale: Idempotent, restartable; ensures accurate counts.
- Alternatives: Row-by-row triggers (slow, complex); single massive query (memory/lock risk).

7. Metrics & Observability
- Decision: Counters for conversations_created_total, conversations_reused_total, conversation_upsert_failures_total; logs include message_id and conversation_key.
- Rationale: Supports reliability SLOs and post-incident analysis.
- Alternatives: Sampled logging only (insufficient for audits); lack of reuse metrics (hard to monitor churn).

## Open Questions (resolved)

- Plus-addressing: Normalize to base (equivalent) — RESOLVED.
- Channels in scope: Email and SMS/MMS only — RESOLVED.

## References

- Feature spec: specs/009-conversation-persistence/spec.md
- Constitution: .specify/memory/constitution.md
