-- Ensure attachment_urls.id has a default sequence for inserts
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.sequences WHERE sequence_name = 'attachment_urls_id_seq'
  ) THEN
    EXECUTE 'CREATE SEQUENCE attachment_urls_id_seq';
  END IF;
  -- Set ownership and default if not set
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'attachment_urls' AND column_name = 'id' AND column_default LIKE 'nextval%'
  ) THEN
    EXECUTE 'ALTER TABLE attachment_urls ALTER COLUMN id SET DEFAULT nextval(''attachment_urls_id_seq'')';
    EXECUTE 'ALTER SEQUENCE attachment_urls_id_seq OWNED BY attachment_urls.id';
  END IF;
  -- Bump sequence to at least max(id)+1
  EXECUTE 'SELECT setval(''attachment_urls_id_seq'', COALESCE((SELECT MAX(id)+1 FROM attachment_urls), 1), false)';
END$$;
