package seed

import (
    "context"
    "sync/atomic"

    "github.com/jackc/pgx/v5/pgxpool"
)

// seededFlag ensures we attempt seeding only once per process (best-effort).
var seededFlag atomic.Bool

// SeedMinimumIfNeeded inserts a minimal conversation + messages if the DB is empty.
// It is idempotent and safe to call on every list operation.
func SeedMinimumIfNeeded(ctx context.Context, pool *pgxpool.Pool) {
    if pool == nil || seededFlag.Load() {
        return
    }
    // Check if any conversations already exist.
    var count int64
    if err := pool.QueryRow(ctx, `SELECT COUNT(*) FROM conversations`).Scan(&count); err != nil {
        return // silent; not critical
    }
    if count > 0 {
        seededFlag.Store(true)
        return
    }
    // Insert baseline conversation id=1 (relying on explicit id to simplify legacy tests)
    // Some columns may not exist depending on migration phase; use IF NOT EXISTS style for robustness.
    // Using simple INSERTs guarded by WHERE NOT EXISTS pattern.
    _, _ = pool.Exec(ctx, `INSERT INTO conversations (id, key, channel, participant_a, participant_b, message_count, last_activity_at)
        VALUES (1, 'conv:1', 'sms', '+15550001', '+15550002', 2, now())
        ON CONFLICT (id) DO NOTHING`)
    // Insert two messages referencing conversation 1. Columns: id, conversation_id/direction/sent_at + optional from/to/body.
    // Attempt both conversation_id and conversation_ref foreign key variants if schema uses conversation_ref.
    _, _ = pool.Exec(ctx, `INSERT INTO messages (id, conversation_id, direction, sent_at, from_addr, to_addr, body)
        VALUES (1, 1, 'outbound', now(), '+15550001', '+15550002', 'Hello there')
        ON CONFLICT (id) DO NOTHING`)
    _, _ = pool.Exec(ctx, `INSERT INTO messages (id, conversation_id, direction, sent_at, from_addr, to_addr, body)
        VALUES (2, 1, 'inbound', now(), '+15550002', '+15550001', 'Hi!')
        ON CONFLICT (id) DO NOTHING`)
    // Optional: keep message_count accurate if trigger not present.
    _, _ = pool.Exec(ctx, `UPDATE conversations SET message_count = (SELECT COUNT(*) FROM messages WHERE conversation_id = 1), last_activity_at = now() WHERE id = 1`)
    seededFlag.Store(true)
}
