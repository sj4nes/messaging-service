-- Audit + normalization triggers (DOWN)

-- Drop normalization triggers
DROP TRIGGER IF EXISTS trg_email_bodies_normalize ON email_bodies;
DROP TRIGGER IF EXISTS trg_xms_bodies_normalize ON xms_bodies;
DROP TRIGGER IF EXISTS trg_phone_numbers_normalize ON phone_numbers;
DROP TRIGGER IF EXISTS trg_email_addresses_normalize ON email_addresses;

-- Drop functions
DROP FUNCTION IF EXISTS normalize_email_body();
DROP FUNCTION IF EXISTS normalize_xms_body();
DROP FUNCTION IF EXISTS normalize_phone_e164();
DROP FUNCTION IF EXISTS normalize_email_lowered();

-- Drop audit triggers
DROP TRIGGER IF EXISTS trg_customers_set_updated_at ON customers;
DROP TRIGGER IF EXISTS trg_providers_set_updated_at ON providers;
DROP TRIGGER IF EXISTS trg_contacts_set_updated_at ON contacts;
DROP TRIGGER IF EXISTS trg_conversations_set_updated_at ON conversations;

DROP FUNCTION IF EXISTS set_updated_at();
