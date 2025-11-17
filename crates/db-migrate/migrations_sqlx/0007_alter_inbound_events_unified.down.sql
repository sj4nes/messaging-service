-- Revert unified metadata additions to inbound_events (DOWN)

ALTER TABLE inbound_events
  DROP COLUMN IF EXISTS channel,
  DROP COLUMN IF EXISTS "from",
  DROP COLUMN IF EXISTS "to",
  DROP COLUMN IF EXISTS provider_message_id,
  DROP COLUMN IF EXISTS processor_id,
  DROP COLUMN IF EXISTS error_code,
  DROP COLUMN IF EXISTS error_message,
  DROP COLUMN IF EXISTS updated_at,
  DROP COLUMN IF EXISTS processed_at;

DROP INDEX IF EXISTS uq_inbound_events_channel_provider_msg;