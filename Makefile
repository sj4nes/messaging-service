.PHONY: setup run test clean help db-up db-down db-logs db-shell build db-reset migrate-status migrate-reset-history

# Load local env vars from .env if present (export to all recipes)
ifneq (,$(wildcard .env))
include .env
export
endif

help:
	@echo "Available commands:"
	@echo "  setup    - Set up the project environment and start database"
	@echo "  run      - Run the application"
	@echo "  run-server - Run the Rust server binary (messaging-server)"
	@echo "  test     - Run tests"
	@echo "  clean    - Clean up temporary files and stop containers"
	@echo "  db-up    - Start the PostgreSQL database"
	@echo "  db-down  - Stop the PostgreSQL database"
	@echo "  db-logs  - Show database logs"
	@echo "  db-shell - Connect to the database shell"
	@echo "  db-reset - Stop containers and remove the database volume (re-runs init.sql on next db-up)"
	@echo "  help     - Show this help message"
	@echo "  lint-shell - Lint bash scripts under .specify/scripts/bash with shellcheck"
	@echo "  migrate-apply - Apply SQLx migrations using db-migrate"
	@echo "  migrate-new   - Create a new timestamped migration file"
	@echo "  migrate-status - Show applied migrations in _sqlx_migrations"
	@echo "  migrate-status-client - Show status via db-migrate (uses DATABASE_URL)"
	@echo "  migrate-reset-history - Truncate _sqlx_migrations (dev only)"

setup: build
	@echo "Setting up the project..."
	@echo "Starting PostgreSQL database..."
	@docker-compose up -d
	@echo "Waiting for database to be ready..."
	@sleep 5
	@echo "Setup complete!"

run:
	@echo "Running the application..."
	@./bin/start.sh

run-server:
	@echo "Running messaging-server..."
	@PORT=$${PORT:-8080} cargo run -p messaging-server

.PHONY: migrate-apply migrate-new
migrate-apply:
	@echo "Applying migrations..."
	@[ -n "$$DATABASE_URL" ] || { echo "DATABASE_URL not set (hint: copy .env.example to .env or set it inline)" >&2; exit 1; }
	@cargo run -p db-migrate -- apply

# Usage: make migrate-new NAME=add_customers_table
migrate-new:
	@name=$${NAME:-new_migration}; echo "Creating migration '$$name'..."; \
	cargo run -p db-migrate -- new "$$name"

.PHONY: migrate-status migrate-reset-history
migrate-status:
	@echo "_sqlx_migrations contents:" \
	&& docker-compose exec -T postgres psql -U messaging_user -d messaging_service -c "SELECT version, description, success, installed_on FROM _sqlx_migrations ORDER BY version;" || true

.PHONY: migrate-status-client
migrate-status-client:
	@echo "db-migrate status (DATABASE_URL):" \
	&& [ -n "$$DATABASE_URL" ] || { echo "DATABASE_URL not set (hint: copy .env.example to .env or set it inline)" >&2; exit 1; } \
	&& cargo run -p db-migrate -- status

migrate-reset-history:
	@echo "WARNING: Truncating _sqlx_migrations (dev only). This will cause all migrations to re-apply." \
	&& docker-compose exec -T postgres psql -U messaging_user -d messaging_service -c "TRUNCATE TABLE _sqlx_migrations;" \
	&& echo "History cleared. Re-run 'make migrate-apply'."

test:
	@echo "Running tests..."
	@echo "Starting test database if not running..."
	@docker-compose up -d
	@echo "Running test script..."
	@./bin/test.sh

clean:
	@echo "Cleaning up..."
	@echo "Stopping and removing containers..."
	@docker-compose down -v
	@echo "Removing any temporary files..."
	@rm -rf *.log *.tmp

db-up:
	@echo "Starting PostgreSQL database..."
	@docker-compose up -d

db-down:
	@echo "Stopping PostgreSQL database..."
	@docker-compose down

db-reset:
	@echo "Stopping containers and removing database volume (all data will be lost)..."
	@docker-compose down -v
	@echo "Next 'make db-up' will reinitialize using init.sql (user & database only). Apply schema via 'make migrate-apply'."

db-logs:
	@echo "Showing database logs..."
	@docker-compose logs -f postgres

db-shell:
	@echo "Connecting to database shell..."
	@docker-compose exec postgres psql -U messaging_user -d messaging_service

.PHONY: lint-shell
lint-shell:
	@command -v shellcheck >/dev/null 2>&1 || { echo "shellcheck not found. Install via 'brew install shellcheck' or your package manager." >&2; exit 1; }
	@shellcheck .specify/scripts/bash/*.sh