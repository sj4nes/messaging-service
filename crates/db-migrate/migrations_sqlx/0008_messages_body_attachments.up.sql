-- Add body text storage and simple attachment URL reference table (UP)
-- Body table allows future parsing/HTML/plain separation; attachment_urls is a minimal placeholder.
CREATE TABLE IF NOT EXISTS message_bodies (
    id BIGSERIAL PRIMARY KEY,
    body TEXT NOT NULL,
    snippet TEXT GENERATED ALWAYS AS (LEFT(body, 160)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS attachment_urls (
    id BIGSERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Link table mapping messages to attachment_urls
CREATE TABLE IF NOT EXISTS message_attachment_urls (
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    attachment_url_id BIGINT NOT NULL REFERENCES attachment_urls(id) ON DELETE CASCADE,
    PRIMARY KEY(message_id, attachment_url_id)
);
