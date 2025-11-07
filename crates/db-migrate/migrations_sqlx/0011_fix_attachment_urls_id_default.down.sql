-- Remove default and sequence ownership (dev only rollback)
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'attachment_urls' AND column_name = 'id' AND column_default LIKE 'nextval%'
  ) THEN
    EXECUTE 'ALTER TABLE attachment_urls ALTER COLUMN id DROP DEFAULT';
  END IF;
  IF EXISTS (
    SELECT 1 FROM information_schema.sequences WHERE sequence_name = 'attachment_urls_id_seq'
  ) THEN
    -- Do not drop the sequence by default to avoid breaking other environments
    NULL;
  END IF;
END$$;
