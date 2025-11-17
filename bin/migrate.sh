#!/usr/bin/env bash
set -euo pipefail

# Simple wrapper around golang-migrate CLI.
# Requires: DATABASE_URL env var, migrate CLI installed (https://github.com/golang-migrate/migrate)
# Default migrations path points to existing SQL files; ensure filenames are compatible with golang-migrate.

MIGRATIONS_DIR=${MIGRATIONS_DIR:-"$(dirname "$0")/../crates/db-migrate/migrations_sqlx"}
DB_URL=${DATABASE_URL:-}
CMD=${1:-up}

if [[ -z "$DB_URL" ]]; then
  echo "ERROR: DATABASE_URL is not set" >&2
  exit 1
fi

if ! command -v migrate >/dev/null 2>&1; then
  echo "ERROR: 'migrate' CLI not found. Install from https://github.com/golang-migrate/migrate" >&2
  exit 1
fi

case "$CMD" in
  up|down|force|version)
    ;;
  *)
    echo "Usage: $0 {up|down|force|version}" >&2
    exit 2
    ;;
fi

# Note: You may need to rename migration files to 000001_xxx.up.sql format for golang-migrate.
# For now, attempt to run as-is.

migrate -path "$MIGRATIONS_DIR" -database "$DB_URL" "$CMD"
