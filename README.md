# Backend Interview Project

This is a scaffold for Hatch's backend interview project. It includes basic setup for development, testing, and deployment.

> **Note on Implementation Language**: This repository contains a **Rust reference implementation** that was created as an initial scaffold. The original interview assignment requirements were later modified to allow **Python, Golang, Elixir, Java, TypeScript/JavaScript, or C/C++**. This Rust codebase serves as a reference architecture and can be used as inspiration, but your solution should be implemented in one of the specified languages. A **Go port** is also available in the `go/` directory as an example of language conversion.

## Prerequisites

Before running this project, ensure you have the following installed:

### Required
- **Docker** 20.10 or higher ([Install Docker](https://docs.docker.com/get-docker/))
- **Docker Compose** 2.x or higher (usually included with Docker Desktop)
- **Minimum 4GB available RAM** (2GB for PostgreSQL, 2GB for application)
- **Git** or **Jujutsu** (jj) for version control

### Optional (for local development of the reference implementations)
- **Rust** latest stable (1.83+) ([Install via rustup](https://rustup.rs/)) - for the Rust reference implementation
- **Go** 1.22+ ([Install Go](https://go.dev/dl/)) - for the Go port
- **cargo** (included with Rust installation)
- **sqlx-cli** (for updating query cache): `cargo install sqlx-cli --no-default-features --features postgres`

**Note**: The `bin/start.sh` script will automatically attempt to install Rust via `make rust-ensure` if not present. However, for containerized deployment via `docker-compose`, Rust is **not required** on the host machine. The Docker build uses latest stable Rust internally. When implementing in other languages, you can use this repository structure and Docker setup as a template.

### First-Time Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd messaging-service
   ```

2. **Copy environment configuration**
   ```bash
   cp .env.example .env
   ```
   Review and adjust `.env` if needed (defaults are suitable for local development).

3. **Choose deployment method**:

   **Option A: Containerized (Recommended for new developers)**
   ```bash
   docker-compose up --build
   ```
   This will:
   - Build the Rust application in a container
   - Start PostgreSQL database
   - Apply database migrations automatically
   - Start the messaging server on http://localhost:8080

   **Option B: Local Rust development**
   ```bash
   make setup    # Starts database and builds Rust binary
   make run      # Runs server locally with cargo
   ```

4. **Verify installation**
   ```bash
   curl http://localhost:8080/healthz
   ```
   Expected response: `{"status":"healthy"}`

5. **Run tests** (optional)
   ```bash
   make test
   # or for containerized:
    docker-compose exec messaging-rust cargo test
   ```

## Guidelines

At Hatch, we work with several message providers to offer a unified way for our Customers to  communicate to their Contacts. Today we offer SMS, MMS, email, voice calls, and voicemail drops. Your task is to implement an HTTP service that supports the core messaging functionality of Hatch, on a much smaller scale. Specific instructions and guidelines on completing the project are below.

### General Guidelines

- You must use one of the following programming languages:
  - **Python**
  - **Golang**
  - **Elixir**
  - **Java**
  - **TypeScript/JavaScript**
  - **C/C++**
- You may use whatever libraries or frameworks you'd like within your chosen language. We strongly encourage you to use whatever you're most familiar with so that you can showcase your skills and know-how.
- You are welcome to use AI, Google, StackOverflow, etc as resources while you're developing. We just ask that you understand the code very well, because we will continue developing on it during your onsite interview.
- For ease of assessment, we strongly encourage you to use the `start.sh` script provided in the `bin/` directory, and implement it to run your service. We will run this script to start your project during our assessment.
- The existing Rust and Go implementations in this repository can serve as reference architectures, but you should implement your solution in one of the languages listed above. 

### Project-specific guidelines

- Assume that a provider may return HTTP error codes like 500, 429 and plan accordingly
- Conversations consist of messages from multiple providers. Feel free to consult providers such as Twilio or Sendgrid docs when designing your solution, but all external resources should be mocked out by your project. We do not expect you to actually integrate with a third party provider as part of this project.
- It's OK to use Google or a coding assistant to produce your code. Just make sure you know it well, because the next step will be to code additional features in this codebase with us during your full interview.

## Requirements

The service should implement:

- **Unified Messaging API**: HTTP endpoints to send and receive messages from both SMS/MMS and Email providers
  - Support sending messages through the appropriate provider based on message type
  - Handle incoming webhook messages from both providers
- **Conversation Management**: Messages should be automatically grouped into conversations based on participants (from/to addresses)
- **Data Persistence**: All conversations and messages must be stored in a relational database with proper relationships and indexing

### Providers

**SMS & MMS**

**Example outbound payload to send an SMS or MMS**

```json
{
    "from": "from-phone-number",
    "to": "to-phone-number",
    "type": "mms" | "sms",
    "body": "text message",
    "attachments": ["attachment-url"] | [] | null,
    "timestamp": "2024-11-01T14:00:00Z" // UTC timestamp
}
```

**Example inbound SMS**

```json
{
    "from": "+18045551234",
    "to": "+12016661234",
    "type": "sms",
    "messaging_provider_id": "message-1",
    "body": "text message",
    "attachments": null,
    "timestamp": "2024-11-01T14:00:00Z" // UTC timestamp
}
```

**Example inbound MMS**

```json
{
    "from": "+18045551234",
    "to": "+12016661234",
    "type": "mms",
    "messaging_provider_id": "message-2",
    "body": "text message",
    "attachments": ["attachment-url"] | [],
    "timestamp": "2024-11-01T14:00:00Z" // UTC timestamp
}
```

**Email Provider**

**Example Inbound Email**

```json
{
    "from": "[user@usehatchapp.com](mailto:user@usehatchapp.com)",
    "to": "[contact@gmail.com](mailto:contact@gmail.com)",
    "xillio_id": "message-2",
    "body": "<html><body>html is <b>allowed</b> here </body></html>",  "attachments": ["attachment-url"] | [],
    "timestamp": "2024-11-01T14:00:00Z" // UTC timestamp
}
```

**Example Email Payload**

```json
{
    "from": "[user@usehatchapp.com](mailto:user@usehatchapp.com)",
    "to": "[contact@gmail.com](mailto:contact@gmail.com)",
    "body": "text message with or without html",
    "attachments": ["attachment-url"] | [],
    "timestamp": "2024-11-01T14:00:00Z" // UTC timestamp
}
```

### Project Structure

This project structure is laid out for you already. You are welcome to move or change things, just update the Makefile, scripts, and/or docker resources accordingly. As part of the evaluation of your code, we will run 

```
.
├── bin/                    # Scripts and executables
│   ├── start.sh           # Application startup script
│   └── test.sh            # API testing script with curl commands
├── docker-compose.yml      # PostgreSQL database setup
├── Makefile               # Build and development commands with docker-compose integration
└── README.md              # This file
```

## Quick Start

For detailed setup instructions, see the **Prerequisites** section above.

### Using Docker Compose (Recommended)
```bash
docker-compose up --build
```
Access the service at http://localhost:8080

### Using Local Development
```bash
make setup  # Initialize database and build
make run    # Start the server
```

### Health Check
```bash
curl -sS http://localhost:8080/healthz | jq
```

### Run Tests
```bash
make test
# or: ./bin/test.sh
```

### Unified Messaging (Feature 006)

- Quickstart (flows, curl examples, runtime config): `specs/006-unified-messaging/quickstart.md`
- OpenAPI contracts for messaging, provider mock, and conversations: `specs/006-unified-messaging/contracts/openapi.yaml`

### PostgreSQL-backed worker (Feature 007)

- When `DATABASE_URL` is set the server uses Postgres-backed persistence and background processing:
    - Inbound webhooks and provider mock inbound insert rows into `inbound_events` (migrations 0006, 0007)
    - Background inbound worker claims events (FOR UPDATE SKIP LOCKED), persists conversations/messages, and marks processed
    - Message bodies are stored in `message_bodies`; attachment URLs in `attachment_urls` and linked via `message_attachment_urls` (migration 0008)
    - Conversations and messages list endpoints read from DB when available and return accurate `meta.total`
    - Fallback to in-memory queue/store when `DATABASE_URL` is unset

    #### Self-testing / In-memory mode (Go)

    The Go server supports a dedicated environment variable to prefer a lightweight in-memory store for self-testing and local development even when a `DATABASE_URL` is present.

    - `GO_API_ENABLE_INMEMORY_FALLBACK=true` — force the Go server to use the in-memory store (useful for fast local runs or deterministic tests that don't require DB persistence). This variable takes precedence over the cross-language `API_ENABLE_INMEMORY_FALLBACK` if set.

    Examples:

    Run the Go server in-memory locally:

    ```bash
    GO_API_ENABLE_INMEMORY_FALLBACK=true make go.run
    ```

    Override Docker compose to use the in-memory mode (not recommended for multi-service integration tests that depend on DB persistence):

    ```yaml
        messaging-go:
            environment:
                GO_API_ENABLE_INMEMORY_FALLBACK: "true"
    ```

    By default the Docker Compose stack uses the database-backed store for the Go service (so messages persist across restarts). If you previously used `INMEMORY_FALLBACK` in your Compose file, prefer `GO_API_ENABLE_INMEMORY_FALLBACK` for an explicit Go-only flag or `API_ENABLE_INMEMORY_FALLBACK` for cross-language parity with the Rust server.

Apply migrations:

```bash
make migrate-apply
```

Apply migrations and seed baseline data:

```bash
make db-seed
```

Or directly:

```bash
cargo run -p db-migrate -- up
```

Default worker config (override via env `API_*`): see `crates/server/config/default.toml` for:
- `worker_batch_size`
- `worker_claim_timeout_secs`
- `worker_max_retries`
- `worker_backoff_base_ms`

### Jujutsu (JJ) Support

This repo supports Jujutsu (JJ) as a first-class VCS. If a `.jj/` directory is present,
spec-kit scripts prefer JJ bookmarks over Git branches. See
`specs/001-jujutsu-scm-support/quickstart.md` for details, numbering rules, and
troubleshooting.

## Development

### Containerized Development (Recommended)
```bash
make docker-up      # Start all services (builds images if needed)
make docker-logs    # View container logs
make docker-down    # Stop all services
make docker-restart # Rebuild and restart
```

### Local Development (requires Rust on host)
```bash
docker-compose up -d postgres  # Start only the database
make run                       # Run server locally with cargo
make test                      # Run tests
make update-agent-context      # Refresh AI agent context files
docker-compose down            # Stop database
```

### Development Tips
- Changes to source code require rebuild: `make docker-up` (rebuilds automatically)
- Database persists across restarts via Docker volume `postgres_data`
- To reset database: `make db-reset` then `make db-up db-seed` to restart and seed baseline data
- **After modifying SQLx queries**: Run `cargo sqlx prepare --workspace` to update offline cache in `.sqlx/` directory (required for Docker builds)

### Contracts

See the health endpoint contract: `specs/002-setup-12fa-server/contracts/openapi.yaml`.

## Database

The application uses PostgreSQL as its database. The docker-compose.yml file sets up:
- PostgreSQL 15 with Alpine Linux
- Database: `messaging_service`
- User: `messaging_user`
- Password: `messaging_password`
- Port: `55432` (host) → `5432` (container)

To connect to the database directly:
```bash
docker-compose exec postgres psql -U messaging_user -d messaging_service
```

From the host (bypassing the container shell), use the mapped host port:
```bash
psql "postgres://messaging_user:messaging_password@localhost:55432/messaging_service" -c 'select 1'
```

Again, you are welcome to make changes here, as long as they're in the docker-compose.yml

### Migrations (SQLx) and Offline Mode

This repo includes a small utility `db-migrate` to manage SQLx migrations:

- Apply pending migrations:
    - `make migrate-apply` (uses `DATABASE_URL` from `.env` if present)
- Apply migrations and seed baseline data:
    - `make db-seed` (runs `migrate-apply` first, then seeds customers/providers/conversations)
- Create a new migration pair:
    - `make migrate-new NAME=add_feature`
- Show applied migrations from inside the container:
    - `make migrate-status`
- Show status via client (which DB am I pointing at?):
    - `make migrate-status-client`

**Note**: The `db-seed` target is automatically run by `make test` to ensure baseline test data exists. For fresh database setups, use `make db-reset db-up db-seed` to reset, start, migrate, and seed in one sequence.

CI tip: set `SQLX_OFFLINE=true` to build without a live database connection when using SQLx elsewhere. Local development can remain online.

## Conversations

Messages are automatically grouped into conversations based on channel and participants. Each conversation:

- Has a **unique key** formatted as `{channel}:{participant_a}<->{participant_b}`
- Maintains **message_count** and **last_activity_at** atomically with each message
- Uses **normalized addresses**:
  - **Email**: Lowercased with plus-tag equivalence (user+tag@example.com → user@example.com)
  - **Phone (SMS/MMS)**: Digits only with optional leading '+' (formatting removed)
- Orders participants lexicographically (participant_a < participant_b)
- Handles concurrent message inserts safely via database constraints

### API Endpoints

- `GET /api/conversations` - List conversations with pagination
  - Returns: id, key, channel, participant_a, participant_b, message_count, last_activity_at
  - Ordered by last_activity_at DESC, id DESC (deterministic)
  
- `GET /api/conversations/{id}/messages` - List messages in a conversation
  - Returns: id, from, to, direction (inbound/outbound), snippet, timestamp
  - Snippets are Unicode-safe (max 64 chars by default, configurable via CONVERSATION_SNIPPET_LENGTH)
  - Ordered chronologically using received_at for inbound, sent_at for outbound

### Configuration

- `CONVERSATION_SNIPPET_LENGTH` - Max characters for message snippets (default: 64, range: 1-4096)

## Observability

Request logging emits: method, path, status, duration_us, client_ip (from `X-Forwarded-For` / `X-Real-IP`), correlation_id (`X-Request-Id` propagated or generated), header_count, and names of sensitive headers (values redacted).

Minimal in-process metrics at `GET /metrics` return JSON counters. For unified messaging, additional dispatch and breaker counters are included:

```json
{
    "ts_unix_ms": 0,
    "rate_limited": 0,
    "breaker_open": 0,
    "dispatch_attempts": 0,
    "dispatch_success": 0,
    "dispatch_rate_limited": 0,
    "dispatch_error": 0,
    "breaker_transitions": 0
}
```

Tracing logs clearly mark provider mocking so you can distinguish real vs simulated flows during development. Look for events prefixed with `mock_...` and the field `mock=true` on records like:

- `mock_inbound` when the mock provider inbound endpoint is hit
- `mock_config_get` / `mock_config_put` when reading/updating mock behavior
- `mock_dispatch_attempt`, `mock_dispatch_outcome`, and `mock_breaker_transition` in the background dispatch worker

### Parity Audit (Feature 012)

The Go Porting Punchlist audit documents behavioral gaps between the Rust reference implementation and the Go port.

Artifacts (specs/012-go-porting-punchlist/):
- `spec.md` – feature specification
- `plan.md` – implementation plan & phases
- `gap-inventory.md` / `gap-inventory.json` – enumerated gaps
- `remediation-tasks.md` – mapping (to be generated)
- `parity-report.json` / `parity-report.md` – closure verification (post‑remediation)

MVP Completion Criteria:
1. Foundational normalization (empty array responses, seed determinism) complete.
2. Gap inventory lists all critical & high gaps with priorities and acceptance criteria.

Audit workflow example:
```bash
bin/test.sh                # run contract tests
cat specs/012-go-porting-punchlist/gap-inventory.md
```

Metrics parity note: `worker_processed` counter currently provisional until async worker implemented.

## Tests

Run HTTP tests:

```bash
bin/test.sh
```

Harness loads cases from `tests/http/tests.json` (validated by `jq`) or falls back to arrays if `jq` is missing.

## Configuration

Core config (port, health path, log level) via env/.env (`crates/core/src/config.rs`). API tunables (body size, rate limits, breaker thresholds) defaulted in `crates/server/src/config.rs`; future work will allow overriding via file/env.
 
### API configuration (T040)

The server loads API-specific limits from `crates/server/config/default.toml` and applies environment overrides. You can point to a different file with `API_CONFIG_FILE`.

Environment overrides (all numbers):

- `API_MAX_BODY_BYTES`
- `API_MAX_ATTACHMENTS`
- `API_RATE_LIMIT_PER_IP_PER_MIN`
- `API_RATE_LIMIT_PER_SENDER_PER_MIN`
- `API_BREAKER_ERROR_THRESHOLD`
- `API_BREAKER_OPEN_SECS`
- `API_PROVIDER_TIMEOUT_PCT`
- `API_PROVIDER_ERROR_PCT`
- `API_PROVIDER_RATELIMIT_PCT`
- `API_PROVIDER_SEED` (optional)

Default file example:

```toml
max_body_bytes = 262144
max_attachments = 8
rate_limit_per_ip_per_min = 120
rate_limit_per_sender_per_min = 60
breaker_error_threshold = 20
breaker_open_secs = 30
provider_timeout_pct = 0
provider_error_pct = 0
provider_ratelimit_pct = 0
# provider_seed = 123456789
```
