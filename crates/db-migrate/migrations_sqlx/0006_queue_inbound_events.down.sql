-- Queue PoC (DOWN)
DROP INDEX IF EXISTS idx_inbound_events_available;
DROP INDEX IF EXISTS idx_inbound_events_status_available;
DROP TABLE IF EXISTS inbound_events;
