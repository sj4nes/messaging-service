-- Extend conversation_messages view: add fallback to message_bodies when normalized dedup body tables are empty
DO $$
BEGIN
  IF to_regclass('public.messages') IS NOT NULL
     AND to_regclass('public.providers') IS NOT NULL THEN
    EXECUTE 'CREATE OR REPLACE VIEW conversation_messages AS
      SELECT
        m.conversation_id,
        m.id AS message_id,
        m.direction,
        m.provider_id,
        m.sent_at,
        m.received_at,
        CASE
          WHEN p.kind = ''email'' AND eb.normalized IS NOT NULL THEN eb.normalized
          WHEN p.kind IN (''sms'',''mms'') AND xb.normalized IS NOT NULL THEN xb.normalized
          WHEN mb.body IS NOT NULL THEN mb.body
          ELSE NULL
        END AS body_text,
        (SELECT COUNT(*) FROM message_attachments ma WHERE ma.message_id = m.id) AS attachments_count
      FROM messages m
      LEFT JOIN providers p ON p.id = m.provider_id
      LEFT JOIN email_bodies eb ON eb.id = m.body_id AND p.kind = ''email''
      LEFT JOIN xms_bodies xb ON xb.id = m.body_id AND p.kind IN (''sms'',''mms'')
      LEFT JOIN message_bodies mb ON mb.id = m.body_id';
  ELSE
    RAISE NOTICE 'Skipping view update (0013): required tables not found';
  END IF;
END$$;
