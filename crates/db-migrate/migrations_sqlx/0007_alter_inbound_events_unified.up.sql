-- Alter inbound_events to support unified processing metadata (UP)

-- Add metadata columns; keep existing attempts/available_at/status as-is for compatibility
ALTER TABLE inbound_events
  ADD COLUMN IF NOT EXISTS channel TEXT NULL,
  ADD COLUMN IF NOT EXISTS "from" TEXT NULL,
  ADD COLUMN IF NOT EXISTS "to" TEXT NULL,
  ADD COLUMN IF NOT EXISTS provider_message_id TEXT NULL,
  ADD COLUMN IF NOT EXISTS processor_id TEXT NULL,
  ADD COLUMN IF NOT EXISTS error_code TEXT NULL,
  ADD COLUMN IF NOT EXISTS error_message TEXT NULL,
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ADD COLUMN IF NOT EXISTS processed_at TIMESTAMPTZ NULL;

-- Idempotency: avoid duplicates by provider id and channel when provider_message_id is present
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_indexes WHERE schemaname = 'public' AND indexname = 'uq_inbound_events_channel_provider_msg'
  ) THEN
    EXECUTE 'CREATE UNIQUE INDEX uq_inbound_events_channel_provider_msg
             ON inbound_events (channel, provider_message_id)
             WHERE provider_message_id IS NOT NULL';
  END IF;
END$$;
