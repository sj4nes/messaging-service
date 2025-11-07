-- Rollback uniqueness constraints
DROP INDEX IF EXISTS ux_message_bodies_body;
DROP INDEX IF EXISTS ux_attachment_urls_url;
