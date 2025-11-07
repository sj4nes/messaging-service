# Backend Interview Project

This is a scaffold for Hatch's backend interview project. It includes basic setup for development, testing, and deployment.

## Guidelines

At Hatch, we work with several message providers to offer a unified way for our Customers to  communicate to their Contacts. Today we offer SMS, MMS, email, voice calls, and voicemail drops. Your task is to implement an HTTP service that supports the core messaging functionality of Hatch, on a much smaller scale. Specific instructions and guidelines on completing the project are below.

### General Guidelines

- You may use whatever programming language, libraries, or frameworks you'd like. 
- We strongly encourage you to use whatever you're most familiar with so that you can showcase your skills and know-how. Candidates will not receive any kind of 'bonus points' or 'red flags' regarding their specific choices of language.
- You are welcome to use AI, Google, StackOverflow, etc as resources while you're developing. We just ask that you understand the code very well, because we will continue developing on it during your onsite interview.
- For ease of assessment, we strongly encourage you to use the `start.sh` script provided in the `bin/` directory, and implement it to run your service. We will run this script to start your project during our assessment. 

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

## Getting Started

1. Clone the repository
2. Run `make setup` to initialize the project
3. Run `docker-compose up -d` to start the PostgreSQL database, or modify it to choose a database of your choice
4. Run `make run` to start the application
        - Or run the Rust server directly: `make run-server`
        - Health check:
            - `curl -sS http://localhost:8080/healthz | jq`
5. Run `make test` to run tests

### Unified Messaging (Feature 006)

- Quickstart (flows, curl examples, runtime config): `specs/006-unified-messaging/quickstart.md`
- OpenAPI contracts for messaging, provider mock, and conversations: `specs/006-unified-messaging/contracts/openapi.yaml`

### PostgreSQL-backed worker (Feature 007)

- Migrations for `inbound_events` extended with processing metadata: `crates/db-migrate/migrations_sqlx/0007_alter_inbound_events_unified.up.sql`
- Server now includes scaffolding modules for DB-backed stores and an inbound worker loop; wiring a PgPool and swapping handlers to persist events are upcoming steps.

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

- Use `docker-compose up -d` to start the PostgreSQL database
- Use `make run` to start the development server
- Use `make test` to run tests
- Use `make update-agent-context` to refresh AI agent context files (e.g., Copilot instructions)
- Use `docker-compose down` to stop the database

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
- Create a new migration pair:
    - `make migrate-new NAME=add_feature`
- Show applied migrations from inside the container:
    - `make migrate-status`
- Show status via client (which DB am I pointing at?):
    - `make migrate-status-client`

CI tip: set `SQLX_OFFLINE=true` to build without a live database connection when using SQLx elsewhere. Local development can remain online.

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
