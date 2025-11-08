-- Remove participant-based conversation columns and related indexes (DOWN)
DROP INDEX IF EXISTS idx_conversations_unique_key;
DROP INDEX IF EXISTS idx_conversations_last_activity;
ALTER TABLE conversations
    DROP COLUMN IF EXISTS channel,
    DROP COLUMN IF EXISTS participant_a,
    DROP COLUMN IF EXISTS participant_b,
    DROP COLUMN IF EXISTS message_count,
    DROP COLUMN IF EXISTS last_activity_at,
    DROP COLUMN IF EXISTS key;
