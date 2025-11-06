-- Bootstrap database user and database for local development.
-- This file is mounted into /docker-entrypoint-initdb.d/ by docker-compose.
-- IMPORTANT: These scripts run only on first initialization of the Postgres data volume.
-- To re-run after initial start, stop containers and remove the named volume (see Makefile clean target).

DO $$
BEGIN
  -- Create role if missing
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'messaging_user') THEN
    CREATE ROLE messaging_user LOGIN PASSWORD 'messaging_password';
  END IF;

  -- Create database if missing, owned by messaging_user
  IF NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'messaging_service') THEN
    CREATE DATABASE messaging_service OWNER messaging_user;
  END IF;
END
$$;

-- Optional: basic privileges; Postgres will set owner on DB creation
GRANT ALL PRIVILEGES ON DATABASE messaging_service TO messaging_user;
