-- Audit + normalization triggers (UP)

-- Minimal audit trigger to keep updated_at current
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach to tables with updated_at
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'customers' AND column_name = 'updated_at') THEN
    CREATE TRIGGER trg_customers_set_updated_at BEFORE UPDATE ON customers FOR EACH ROW EXECUTE FUNCTION set_updated_at();
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'providers' AND column_name = 'updated_at') THEN
    CREATE TRIGGER trg_providers_set_updated_at BEFORE UPDATE ON providers FOR EACH ROW EXECUTE FUNCTION set_updated_at();
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'contacts' AND column_name = 'updated_at') THEN
    CREATE TRIGGER trg_contacts_set_updated_at BEFORE UPDATE ON contacts FOR EACH ROW EXECUTE FUNCTION set_updated_at();
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'conversations' AND column_name = 'updated_at') THEN
    CREATE TRIGGER trg_conversations_set_updated_at BEFORE UPDATE ON conversations FOR EACH ROW EXECUTE FUNCTION set_updated_at();
  END IF;
END$$;

-- Normalization helpers
CREATE OR REPLACE FUNCTION normalize_email_body() RETURNS TRIGGER AS $$
BEGIN
  IF NEW.normalized IS NULL THEN
    NEW.normalized := lower(trim(NEW.raw));
  END IF;
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION normalize_xms_body() RETURNS TRIGGER AS $$
BEGIN
  IF NEW.normalized IS NULL THEN
    NEW.normalized := regexp_replace(trim(NEW.raw), '\\s+', ' ', 'g');
  END IF;
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION normalize_phone_e164() RETURNS TRIGGER AS $$
BEGIN
  IF NEW.e164 IS NULL OR NEW.e164 = '' THEN
    NEW.e164 := regexp_replace(NEW.raw, '[^0-9]+', '', 'g');
  END IF;
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION normalize_email_lowered() RETURNS TRIGGER AS $$
BEGIN
  IF NEW.lowered IS NULL THEN
    NEW.lowered := lower(trim(NEW.raw));
  END IF;
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

-- Attach normalization triggers guarded by existence
DO $$
BEGIN
  IF to_regclass('public.email_bodies') IS NOT NULL THEN
    EXECUTE 'CREATE TRIGGER trg_email_bodies_normalize BEFORE INSERT ON email_bodies FOR EACH ROW EXECUTE FUNCTION normalize_email_body()';
  END IF;
  IF to_regclass('public.xms_bodies') IS NOT NULL THEN
    EXECUTE 'CREATE TRIGGER trg_xms_bodies_normalize BEFORE INSERT ON xms_bodies FOR EACH ROW EXECUTE FUNCTION normalize_xms_body()';
  END IF;
  IF to_regclass('public.phone_numbers') IS NOT NULL THEN
    EXECUTE 'CREATE TRIGGER trg_phone_numbers_normalize BEFORE INSERT ON phone_numbers FOR EACH ROW EXECUTE FUNCTION normalize_phone_e164()';
  END IF;
  IF to_regclass('public.email_addresses') IS NOT NULL THEN
    EXECUTE 'CREATE TRIGGER trg_email_addresses_normalize BEFORE INSERT ON email_addresses FOR EACH ROW EXECUTE FUNCTION normalize_email_lowered()';
  END IF;
END$$;
