-- Indexes (UP)
DO $$
BEGIN
  IF to_regclass('public.messages') IS NOT NULL THEN
    EXECUTE 'CREATE INDEX IF NOT EXISTS idx_messages_conversation_sent_at ON messages (conversation_id, sent_at DESC)';
  END IF;
  IF to_regclass('public.conversation_participants') IS NOT NULL THEN
    EXECUTE 'CREATE INDEX IF NOT EXISTS idx_conversation_participants_contact ON conversation_participants (contact_id)';
  END IF;
  IF to_regclass('public.endpoint_mappings') IS NOT NULL THEN
    EXECUTE 'CREATE INDEX IF NOT EXISTS idx_endpoint_mappings_ckh ON endpoint_mappings (customer_id, kind, hash)';
  END IF;
END$$;
