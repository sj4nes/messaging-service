-- Drop conversation-based foreign key column from messages (DOWN)
DROP INDEX IF EXISTS idx_messages_conversation_ref;
ALTER TABLE messages DROP COLUMN IF EXISTS conversation_ref;
