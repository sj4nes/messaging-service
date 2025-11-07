-- Revert link table and url column (DOWN)
DROP TABLE IF EXISTS message_attachment_urls;
-- Be cautious dropping column as it may be in use; only drop if present
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name='attachment_urls' AND column_name='url'
  ) THEN
    EXECUTE 'ALTER TABLE attachment_urls DROP COLUMN url';
  END IF;
END$$;
