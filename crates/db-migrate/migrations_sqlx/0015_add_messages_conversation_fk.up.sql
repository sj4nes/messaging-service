-- Add conversation-based foreign key to messages (UP)
ALTER TABLE messages ADD COLUMN IF NOT EXISTS conversation_ref BIGINT NULL;

-- Backfill placeholder: set conversation_ref to existing conversation_id if present (legacy naming)
-- (Adjust if legacy schema differs)
UPDATE messages SET conversation_ref = conversation_id WHERE conversation_ref IS NULL;

-- Ensure not null moving forward once backfill complete (will finalize in later migration phase)
-- For now keep nullable to allow phased rollout.

-- Supporting index
DO $$
BEGIN
  IF to_regclass('public.idx_messages_conversation_ref') IS NULL THEN
    EXECUTE 'CREATE INDEX idx_messages_conversation_ref ON messages (conversation_ref)';
  END IF;
END$$;
