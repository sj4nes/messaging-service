-- Add participant-based conversation columns and indexes (UP)
ALTER TABLE conversations
    ADD COLUMN IF NOT EXISTS channel TEXT NULL,
    ADD COLUMN IF NOT EXISTS participant_a TEXT NULL,
    ADD COLUMN IF NOT EXISTS participant_b TEXT NULL,
    ADD COLUMN IF NOT EXISTS message_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_activity_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ADD COLUMN IF NOT EXISTS key TEXT NULL;

-- Unique index on (channel, participant_a, participant_b) for non-null rows
DO $$
BEGIN
  IF to_regclass('public.idx_conversations_unique_key') IS NULL THEN
    EXECUTE 'CREATE UNIQUE INDEX idx_conversations_unique_key ON conversations (channel, participant_a, participant_b) WHERE channel IS NOT NULL AND participant_a IS NOT NULL AND participant_b IS NOT NULL';
  END IF;
END$$;

-- Last activity index for deterministic paging
DO $$
BEGIN
  IF to_regclass('public.idx_conversations_last_activity') IS NULL THEN
    EXECUTE 'CREATE INDEX idx_conversations_last_activity ON conversations (last_activity_at DESC)';
  END IF;
END$$;
