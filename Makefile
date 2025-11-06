.PHONY: setup run test clean help db-up db-down db-logs db-shell build

build:
	@echo "Building the project..."
	@cargo build

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
	@echo "  help     - Show this help message"
	@echo "  lint-shell - Lint bash scripts under .specify/scripts/bash with shellcheck"

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