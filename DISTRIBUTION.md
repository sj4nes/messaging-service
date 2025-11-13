# Distribution Guide

This document explains how to package and distribute the messaging-service to other developers.

## For Distributors

### Prerequisites for Creating Distribution

Before creating an archive, ensure SQLx offline query cache is up-to-date:

```bash
# 1. Start database
docker-compose up -d postgres

# 2. Apply all migrations
export DATABASE_URL="postgres://messaging_user:messaging_password@localhost:55432/messaging_service"
cargo run -p db-migrate -- apply

# 3. Generate SQLx offline data (required for Docker build without database)
cargo sqlx prepare --workspace

# 4. Commit .sqlx/ directory to version control
git add .sqlx/
git commit -m "Update SQLx offline query cache"
```

### Creating an Archive

```bash
# From repository root
git archive --format=zip --output=messaging-service.zip HEAD
# or with jj:
jj git export && git archive --format=zip --output=messaging-service.zip HEAD
```

**Important**: The archive must include the `.sqlx/` directory for Docker builds to work without a database connection during compilation.

### What's Included

The archive contains:
- **Source code** (Rust workspace with 3 crates)
- **Dockerfile** (multi-stage build for messaging-server)
- **docker-compose.yml** (full stack: app + PostgreSQL)
- **Makefile** (development shortcuts)
- **bin/start.sh** (with automatic Rust installation)
- **.env.example** (configuration template)
- **Database migrations** (SQLx migrations in crates/db-migrate)

## For Recipients (New Developers)

### Requirements

- **Docker** 20.10+ with **Docker Compose** 2.x
- **4GB RAM** minimum (2GB PostgreSQL + 2GB application)
- **No Rust required** (built inside Docker container)

### Quick Start (Containerized - Recommended)

```bash
# 1. Extract archive
unzip messaging-service.zip -d messaging-service
cd messaging-service

# 2. Copy environment template
cp .env.example .env

# 3. Start everything
docker-compose up --build

# 4. Verify (in another terminal)
curl http://localhost:8080/healthz
# Expected: {"status":"healthy"}

# 5. Run tests
docker-compose exec messaging-rust cargo test
```

That's it! The system is now running with:
- PostgreSQL on port 55432 (mapped from internal 5432)
- HTTP server on port 8080
- All migrations applied automatically

### Alternative: Local Development (Requires Rust)

If you prefer to develop locally without containers:

```bash
# 1. Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# 2. Start database only
docker-compose up -d postgres

# 3. Run server locally
cp .env.example .env
make run
# or: ./bin/start.sh
```

The `bin/start.sh` script will automatically install Rust if missing via `make rust-ensure`.

### Common Tasks

```bash
# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Rebuild after code changes
docker-compose up --build

# Reset database (WARNING: deletes all data)
docker-compose down -v
docker-compose up --build

# Run HTTP tests
docker-compose exec messaging-rust /bin/bash -c "cd /build && ./bin/test.sh"

# Check migration status
docker-compose exec messaging-rust cargo run -p db-migrate -- status
```

### Architecture Overview

```
messaging-service/
├── crates/
│   ├── core/           # Business logic (conversations, normalization)
│   ├── server/         # HTTP API (Axum + Tokio)
│   └── db-migrate/     # Database migrations (SQLx)
├── tests/
│   ├── integration/    # Integration tests (conversation flows)
│   ├── contract/       # API contract tests
│   └── unit/           # Unit tests
├── Dockerfile          # Multi-stage Rust build
└── docker-compose.yml  # PostgreSQL + messaging-rust
```

### Environment Variables

Key configuration (see `.env.example`):

```bash
PORT=8080                    # HTTP server port
DATABASE_URL=postgres://...  # PostgreSQL connection string
LOG_LEVEL=info              # Logging verbosity (trace|debug|info|warn|error)
HEALTH_PATH=/healthz        # Health check endpoint
```

Additional config via `API_*` environment variables (see README.md "Configuration" section).

### Troubleshooting

**Q: Port 8080 already in use**  
A: Change `PORT=8081` in `.env` and update `docker-compose.yml` ports mapping.

**Q: Database connection failed**  
A: Ensure PostgreSQL container is healthy: `docker-compose ps`  
Check logs: `docker-compose logs postgres`

**Q: Build fails with "cargo not found"**  
A: This should not happen in containerized mode. If using local development, install Rust: https://rustup.rs/

**Q: Tests fail with "database does not exist"**  
A: Migrations may not have run. Apply manually:  
`docker-compose exec messaging-rust cargo run -p db-migrate -- apply`

**Q: How do I see what migrations have been applied?**  
A: `docker-compose exec postgres psql -U messaging_user -d messaging_service -c "SELECT * FROM _sqlx_migrations;"`

### Performance Benchmarks

Expected performance (local development MacBook Pro M1):
- Conversation upsert: P95 < 10ms
- 100 concurrent inserts: 1 conversation created (race-safe)
- Load test 1000 requests: ~200-300 RPS

Run load tests:
```bash
docker-compose exec messaging-rust cargo test --test load_report -- --nocapture
```

### Support

For issues or questions:
1. Check logs: `docker-compose logs -f`
2. Review API documentation: `specs/006-unified-messaging/contracts/openapi.yaml`
3. See conversation quickstart: `specs/009-conversation-persistence/quickstart.md`
