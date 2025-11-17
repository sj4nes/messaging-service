-- Ensure attachment URL schema matches code expectations (UP)
-- Add missing url column if the table exists but lacks it; create link table if missing.

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name='attachment_urls' AND column_name='url'
  ) THEN
    EXECUTE 'ALTER TABLE attachment_urls ADD COLUMN url TEXT NOT NULL DEFAULT ''''';
    -- Optional: drop default after backfill/compat
    EXECUTE 'ALTER TABLE attachment_urls ALTER COLUMN url DROP DEFAULT';
  END IF;
END$$;

CREATE TABLE IF NOT EXISTS message_attachment_urls (
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    attachment_url_id BIGINT NOT NULL REFERENCES attachment_urls(id) ON DELETE CASCADE,
    PRIMARY KEY(message_id, attachment_url_id)
);
