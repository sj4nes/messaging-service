-- Deduplicate existing rows before adding unique index
BEGIN;

-- 1) Remap messages.body_id to the lowest id per body
WITH dedup AS (
	SELECT body, MIN(id) AS keep_id, ARRAY_AGG(id) AS all_ids
	FROM message_bodies
	GROUP BY body
	HAVING COUNT(*) > 1
), map AS (
	SELECT UNNEST(all_ids) AS old_id, keep_id FROM dedup
)
UPDATE messages m
SET body_id = map.keep_id
FROM map
WHERE m.body_id = map.old_id AND m.body_id IS NOT NULL;

-- 2) Delete duplicate message_bodies rows, keep the lowest id per body
DELETE FROM message_bodies mb
USING (
	SELECT id FROM (
		SELECT id, ROW_NUMBER() OVER (PARTITION BY body ORDER BY id) AS rn
		FROM message_bodies
	) t
	WHERE t.rn > 1
) d
WHERE mb.id = d.id;

-- 3) Add unique indexes to enforce dedup going forward
CREATE UNIQUE INDEX IF NOT EXISTS ux_message_bodies_body ON message_bodies (body);

-- Also deduplicate attachment URLs to avoid duplicate rows
-- First, delete duplicates keeping the lowest id
DELETE FROM attachment_urls a
USING (
	SELECT id FROM (
		SELECT id, ROW_NUMBER() OVER (PARTITION BY url ORDER BY id) AS rn
		FROM attachment_urls
	) t
	WHERE t.rn > 1
) d
WHERE a.id = d.id;

CREATE UNIQUE INDEX IF NOT EXISTS ux_attachment_urls_url ON attachment_urls (url);

COMMIT;
