.PHONY: setup run test clean help db-up db-down db-logs db-shell build db-reset migrate-status migrate-reset-history \
	dx-setup py-venv py-install-jsonschema validate-events rust-check rust-install rust-ensure rust-version \
	go-ensure go-install \
	prereqs-check prereqs-install \
	sqlc-install \
	docker-build docker-up docker-down docker-logs docker-restart go.tidy go.build go.test go.run go.docker-build go.sqlc lint lint-shell update-agent-context

.DEFAULT_GOAL := help

# --- Go targets ---
GO       ?= go
GO_DIR   ?= go

.PHONY: go.tidy
go.tidy:
	cd $(GO_DIR) && $(GO) mod tidy

.PHONY: go.build
go.build:
	cd $(GO_DIR) && $(GO) build ./...

.PHONY: go.tidy
.PHONY: go.test
go.test:
	cd $(GO_DIR) && $(GO) test ./...

.PHONY: go.run
go.run:
	cd $(GO_DIR) && $(GO) run ./cmd/server

.PHONY: go.docker-build
go.docker-build:
	docker build -f $(GO_DIR)/Dockerfile -t messaging-go:dev $(GO_DIR)

.PHONY: go.sqlc
go.sqlc:
	@command -v sqlc >/dev/null 2>&1 || { echo "sqlc not found; running 'make sqlc-install'"; $(MAKE) sqlc-install; }
	cd $(GO_DIR) && sqlc generate

.PHONY: sqlc-install
sqlc-install:
	@echo "Installing sqlc (macOS/Homebrew)..."
	@command -v brew >/dev/null 2>&1 || { echo "Homebrew not found; please install it from https://brew.sh/" >&2; exit 1; }
	@brew install sqlc || true
	@echo "sqlc installation attempted; verify with: sqlc version"
# Load local env vars from .env if present (export to all recipes)
ifneq (,$(wildcard .env))
include .env
export
endif


help:
	@echo "Available commands:"
	@echo "  setup                - Set up the project environment and start database"
	@echo "  run                  - Run the application via ./bin/start.sh"
	@echo "  run-server           - Run the Rust server binary (messaging-server)"
	@echo "  dev                  - Run server with auto-reload if cargo-watch is available"
	@echo "  build                - cargo build --all (debug)"
	@echo "  build-rust           - Build the Rust server"
	@echo "  build-release        - cargo build --release --all"
	@echo "  go.build-release     - Build Go server in release mode"
	@echo "  test                 - Run tests (starts docker-compose and ./bin/test.sh)"
	@echo "  lint                 - Run cargo fmt and clippy on the server crate"
	@echo "  clean                - Clean up temporary files and stop containers"
	@echo ""
	@echo "Database targets:"
	@echo "  db-up                - Start the PostgreSQL database via docker-compose"
	@echo "  db-down              - Stop the PostgreSQL database"
	@echo "  db-reset             - Stop containers and remove the database volume (re-runs init.sql on next db-up)"
	@echo "  db-logs              - Show database logs"
	@echo "  db-shell             - Connect to the database shell"
	@echo ""
	@echo "Migration targets (SQLx/db-migrate):"
	@echo "  migrate-apply        - Apply SQLx migrations using db-migrate (requires DATABASE_URL)"
	@echo "  migrate-new          - Create a new timestamped migration file (NAME=...)"
	@echo "  migrate-status       - Show applied migrations in _sqlx_migrations via docker-compose exec"
	@echo "  migrate-status-client- Show status via db-migrate (uses DATABASE_URL)"
	@echo "  migrate-reset-history- Truncate _sqlx_migrations (dev only)"
	@echo ""
	@echo "Developer experience targets:"
	@echo "  dx-setup             - Bootstrap DX tools (Python venv + jsonschema, ensure Rust toolchain)"
	@echo "  update-agent-context - Update AI agent context files (e.g., Copilot instructions)"
	@echo "  validate-events      - Validate event examples against the envelope schema (uses .venv)"
	@echo "  py-venv              - Create Python virtual environment at .venv (and upgrade pip)"
	@echo "  py-install-jsonschema- Install/upgrade jsonschema in the .venv"
	@echo "  go-ensure            - Ensure Go is installed on developer machine (Homebrew macOS)"
	@echo "  go-install           - Install Go via Homebrew (macOS)"
	@echo "  prereqs-check        - Check required developer tools are installed"
	@echo "  prereqs-install      - Install developer tools (macOS/Homebrew)"
	@echo "  rust-check           - Check that cargo is installed and print cargo version"
	@echo "  rust-install         - Install Rust toolchain via rustup (non-interactive)"
	@echo "  rust-ensure          - Ensure Rust toolchain is available (installs if missing)"
	@echo "  lint-shell           - Lint bash scripts under .specify/scripts/bash with shellcheck"
	@echo ""
	@echo "Go targets:"
	@echo "  go.tidy              - Run 'go mod tidy' in the Go module"
	@echo "  go.build             - Build all Go packages"
	@echo "  go.test              - Run Go tests"
	@echo "  go.run               - Run the Go server (./cmd/server)"
	@echo "  go.docker-build      - Build the Go service Docker image (messaging-go:dev)"
	@echo "  go.sqlc              - Generate Go code from SQL schemas using sqlc"
	@echo "  sqlc-install         - Install sqlc via Homebrew (macOS)"
	@echo "  cargo-sqlx           - Install SQLx CLI (cargo install sqlx-cli)"
	@echo ""
	@echo "Docker Compose targets:"
	@echo "  docker-build         - Build Docker images for the application"
	@echo "  docker-up            - Start all services in containers (builds if needed)"
	@echo "  docker-down          - Stop and remove all containers"
	@echo "  docker-logs          - Show logs from all containers"
	@echo "  docker-restart       - Restart all containers"

build:
	@$(MAKE) build-rust
	@$(MAKE) build-go

build-release:
	@$(MAKE) build-rust-release
	@$(MAKE) go.build-release

.PHONY: build-rust build-rust-release build-go
build-go: go.sqlc go.build
build-rust:
	@echo "Generating SQLx offline data (if sqlx CLI is present)"
	@command -v cargo-sqlx >/dev/null 2>&1 && cargo sqlx prepare --workspace || true
	@cargo build --all

build-rust-release:
	@cargo build --release --all

.PHONY: go.build-release
go.build-release: go.sqlc
	@echo "Building Go (release)"
	cd $(GO_DIR) && $(GO) build -ldflags "-s -w" ./...


setup: dx-setup build
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
	@echo "Applying migrations via Go migrate helper..."
	@[ -n "$$DATABASE_URL" ] || { echo "DATABASE_URL not set (hint: copy .env.example to .env or set it inline)" >&2; exit 1; }
	@cd $(GO_DIR) && MIGRATIONS_DIR="../crates/db-migrate/migrations_sqlx" DATABASE_URL="$$DATABASE_URL" $(GO) run ./cmd/migrate

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

dx-setup: py-install-jsonschema rust-ensure go-ensure
	@echo "DX setup complete. To run validation: make validate-events"

.PHONY: py-venv
py-venv:
	@command -v python3 >/dev/null 2>&1 || { echo "python3 not found on PATH" >&2; exit 1; }
	@[ -d .venv ] || { echo "Creating Python venv at .venv"; python3 -m venv .venv; }
	@.venv/bin/python -m pip install --upgrade pip >/dev/null
	@echo "Python venv ready: .venv"


.PHONY: prereqs-check prereqs-install FORCE
prereqs-check: FORCE
		@echo "Checking common developer prerequisites..."
		@missing=0; \
		command -v docker >/dev/null 2>&1 || { echo "  docker: NOT FOUND"; missing=1; }; \
		command -v docker-compose >/dev/null 2>&1 || { echo "  docker-compose: NOT FOUND"; missing=1; }; \
		command -v cargo >/dev/null 2>&1 || { echo "  cargo (Rust): NOT FOUND (run make rust-install)"; missing=1; }; \
		command -v go >/dev/null 2>&1 || { echo "  go: NOT FOUND (install Go from https://go.dev/dl/)"; missing=1; }; \
		command -v sqlc >/dev/null 2>&1 || { echo "  sqlc: NOT FOUND (see https://docs.sqlc.dev/)"; missing=1; }; \
		command -v migrate >/dev/null 2>&1 || { echo "  migrate: NOT FOUND (install golang-migrate: brew install golang-migrate)"; missing=1; }; \
		command -v jq >/dev/null 2>&1 || { echo "  jq: NOT FOUND (used by tests)"; missing=1; }; \
		command -v shellcheck >/dev/null 2>&1 || { echo "  shellcheck: NOT FOUND (used by lint-shell)"; missing=1; }; \
		command -v python3 >/dev/null 2>&1 || { echo "  python3: NOT FOUND"; missing=1; }; \
		if [ $$missing -eq 1 ]; then echo "Some dependencies are missing. Run 'make prereqs-install' (macOS) or install manually." >&2; exit 1; fi; \
		echo "All prerequisites appear to be present."


prereqs-install: FORCE
		@echo "Attempting to install basic prerequisites (macOS/Homebrew)"
		@command -v brew >/dev/null 2>&1 || { echo "Homebrew not found; please install it from https://brew.sh/" >&2; exit 1; }
		@brew install sqlc golang-migrate jq shellcheck || true
		@$(MAKE) py-install-jsonschema || true
		# Optionally: install SQLx CLI via cargo for offline prepare
		@command -v cargo >/dev/null 2>&1 && cargo install sqlx-cli --no-default-features --features postgres || true
		@$(MAKE) rust-ensure || true
		@echo "Prereqs install attempted. Verify with 'make prereqs-check'."

.PHONY: py-install-jsonschema
py-install-jsonschema: py-venv
	@echo "Installing/refreshing jsonschema in .venv..."
	@.venv/bin/pip install -q --upgrade jsonschema
	@echo "jsonschema ready."


FORCE:

.PHONY: validate-events
validate-events: py-install-jsonschema
	@echo "Validating event examples against envelope schema..."
	@.venv/bin/python specs/004-create-domain-events/contracts/events/validate_examples.py

.PHONY: rust-check
rust-check:
	@command -v cargo >/dev/null 2>&1 && echo "cargo present: $$(cargo --version)" || { echo "cargo not found" >&2; exit 1; }

.PHONY: go-ensure go-install
go-ensure:
	@command -v go >/dev/null 2>&1 || { echo "go not found; running 'make go-install'"; $(MAKE) go-install; }
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
