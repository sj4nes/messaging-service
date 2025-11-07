-- Revert conversation_messages view to placeholder body_text (clean follow-up DOWN)
DO $$
BEGIN
  IF to_regclass('public.messages') IS NOT NULL THEN
    EXECUTE 'CREATE OR REPLACE VIEW conversation_messages AS
      SELECT
        m.conversation_id,
        m.id AS message_id,
        m.direction,
        m.provider_id,
        m.sent_at,
        m.received_at,
        NULL::TEXT AS body_text,
        (SELECT COUNT(*) FROM message_attachments ma WHERE ma.message_id = m.id) AS attachments_count
      FROM messages m';
  ELSE
    RAISE NOTICE 'Skipping view revert (0012): messages table not found';
  END IF;
END$$;
