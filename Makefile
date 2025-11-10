.PHONY: setup run test clean help db-up db-down db-logs db-shell build db-reset migrate-status migrate-reset-history \
	dx-setup py-venv py-install-jsonschema validate-events rust-check rust-install rust-ensure rust-version \
	docker-build docker-up docker-down docker-logs docker-restart

# --- Go targets ---
GO       ?= go
GO_DIR   ?= go

.PHONY: go.tidy
go.tidy:
	cd $(GO_DIR) && $(GO) mod tidy

.PHONY: go.build
go.build:
	cd $(GO_DIR) && $(GO) build ./...

.PHONY: go.test
go.test:
	cd $(GO_DIR) && $(GO) test ./...

.PHONY: go.run
go.run:
	cd $(GO_DIR) && $(GO) run ./cmd/server

.PHONY: go.docker-build
go.docker-build:
	docker build -f $(GO_DIR)/Dockerfile -t messaging-go:dev $(GO_DIR)
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
	@echo "  dev      - Run server with auto-reload if cargo-watch is available"
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
	@echo "  dx-setup - Bootstrap DX tools (Python venv + jsonschema, ensure Rust toolchain)"
	@echo "  update-agent-context - Update AI agent context files (e.g., Copilot instructions)"
	@echo "  validate-events - Validate event examples against the envelope schema (uses .venv)"
	@echo "  py-venv - Create Python virtual environment at .venv (and upgrade pip)"
	@echo "  rust-ensure - Install Rust via rustup if cargo is not present"
	@echo ""
	@echo "Docker Compose targets:"
	@echo "  docker-build   - Build Docker images for the application"
	@echo "  docker-up      - Start all services in containers (builds if needed)"
	@echo "  docker-down    - Stop and remove all containers"
	@echo "  docker-logs    - Show logs from all containers"
	@echo "  docker-restart - Restart all containers"

build:
	@cargo build --all

build-release:
	@cargo build --release --all


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

.PHONY: dev
dev:
	@echo "Starting messaging-server in watch mode (if cargo-watch is installed)..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		PORT=$${PORT:-8080} cargo watch -x 'run -p messaging-server'; \
	else \
		echo "cargo-watch not found. Install with: cargo install cargo-watch"; \
		echo "Falling back to normal run (no auto-reload)..."; \
		PORT=$${PORT:-8080} cargo run -p messaging-server; \
	fi

.PHONY: lint
lint:
	@command -v cargo >/dev/null 2>&1 || { echo "cargo not found; run 'make rust-ensure'" >&2; exit 1; }
	@echo "Running fmt..."
	@cargo fmt --all
	@echo "Running clippy on server crate..."
	@cargo clippy -p messaging-server -- -W clippy::all || true

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

# -----------------------------
# Developer Experience (DX)
# -----------------------------

dx-setup: py-install-jsonschema rust-ensure
	@echo "DX setup complete. To run validation: make validate-events"

.PHONY: py-venv
py-venv:
	@command -v python3 >/dev/null 2>&1 || { echo "python3 not found on PATH" >&2; exit 1; }
	@[ -d .venv ] || { echo "Creating Python venv at .venv"; python3 -m venv .venv; }
	@.venv/bin/python -m pip install --upgrade pip >/dev/null
	@echo "Python venv ready: .venv"

.PHONY: py-install-jsonschema
py-install-jsonschema: py-venv
	@echo "Installing/refreshing jsonschema in .venv..."
	@.venv/bin/pip install -q --upgrade jsonschema
	@echo "jsonschema ready."

.PHONY: validate-events
validate-events: py-install-jsonschema
	@echo "Validating event examples against envelope schema..."
	@.venv/bin/python specs/004-create-domain-events/contracts/events/validate_examples.py

.PHONY: rust-check
rust-check:
	@command -v cargo >/dev/null 2>&1 && echo "cargo present: $$(cargo --version)" || { echo "cargo not found" >&2; exit 1; }

.PHONY: rust-install
rust-install:
	@echo "Installing Rust toolchain via rustup (non-interactive)..."
	@curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	@# Attempt to load cargo into current shell for subsequent commands
	@{ [ -f $$HOME/.cargo/env ] && . $$HOME/.cargo/env; } >/dev/null 2>&1 || true
	@{ command -v cargo >/dev/null 2>&1 && echo "cargo installed: $$(cargo --version)"; } || echo "Rust installed. Restart your shell to use cargo."

.PHONY: rust-ensure
rust-ensure:
	@if command -v cargo >/dev/null 2>&1; then \
		echo "cargo present: $$(cargo --version)"; \
	else \
		$(MAKE) rust-install; \
	fi

.PHONY: update-agent-context
update-agent-context:
	@echo "Updating agent context files..."
	@.specify/scripts/bash/update-agent-context.sh

# -----------------------------
# Docker Compose Targets
# -----------------------------

.PHONY: docker-build
docker-build:
	@echo "Building Docker images..."
	@docker-compose build

.PHONY: docker-up
docker-up:
	@echo "Starting all services via Docker Compose..."
	@docker-compose up --build -d
	@echo "Services starting. Check status with: docker-compose ps"
	@echo "View logs with: make docker-logs"

.PHONY: docker-down
docker-down:
	@echo "Stopping all Docker Compose services..."
	@docker-compose down

.PHONY: docker-logs
docker-logs:
	@echo "Showing logs from all containers (Ctrl+C to exit)..."
	@docker-compose logs -f

.PHONY: docker-restart
docker-restart: docker-down docker-up
	@echo "All services restarted"