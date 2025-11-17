-- Views (UP)
DO $$
BEGIN
  IF to_regclass('public.conversations') IS NOT NULL AND to_regclass('public.messages') IS NOT NULL THEN
    EXECUTE 'CREATE OR REPLACE VIEW conversation_overview AS
      SELECT
        c.id AS conversation_id,
        c.customer_id,
        MAX(COALESCE(m.received_at, m.sent_at)) AS last_message_at,
        COUNT(m.id) AS message_count,
        (SELECT COUNT(DISTINCT cp.contact_id) FROM conversation_participants cp WHERE cp.conversation_id = c.id) AS participant_count
      FROM conversations c
      LEFT JOIN messages m ON m.conversation_id = c.id
      GROUP BY c.id, c.customer_id';

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
    RAISE NOTICE 'Skipping view creation: base tables not found';
  END IF;
END$$;
