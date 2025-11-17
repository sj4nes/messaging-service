-- Queue PoC (UP)
CREATE TABLE IF NOT EXISTS inbound_events (
    id BIGSERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    available_at TIMESTAMPTZ NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('pending','processing','done','dead'))
);

-- Visibility/dispatch indexes
CREATE INDEX IF NOT EXISTS idx_inbound_events_status_available ON inbound_events (status, available_at);
CREATE INDEX IF NOT EXISTS idx_inbound_events_available ON inbound_events (available_at);
