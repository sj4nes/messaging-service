-- Revert conversation_messages view to version from 0012 (without message_bodies fallback)
DO $$
BEGIN
  IF to_regclass('public.messages') IS NOT NULL
     AND to_regclass('public.providers') IS NOT NULL
     AND to_regclass('public.email_bodies') IS NOT NULL
     AND to_regclass('public.xms_bodies') IS NOT NULL THEN
    EXECUTE 'CREATE OR REPLACE VIEW conversation_messages AS
      SELECT
        m.conversation_id,
        m.id AS message_id,
        m.direction,
        m.provider_id,
        m.sent_at,
        m.received_at,
        CASE
          WHEN p.kind = ''email'' THEN eb.normalized
          WHEN p.kind IN (''sms'',''mms'') THEN xb.normalized
          ELSE NULL
        END AS body_text,
        (SELECT COUNT(*) FROM message_attachments ma WHERE ma.message_id = m.id) AS attachments_count
      FROM messages m
      LEFT JOIN providers p ON p.id = m.provider_id
      LEFT JOIN email_bodies eb ON eb.id = m.body_id AND p.kind = ''email''
      LEFT JOIN xms_bodies xb ON xb.id = m.body_id AND p.kind IN (''sms'',''mms'')';
  ELSE
    RAISE NOTICE 'Skipping view revert (0013 down): required tables not found';
  END IF;
END$$;
