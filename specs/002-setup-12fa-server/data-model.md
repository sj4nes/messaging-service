# Data Model — Setup 12‑Factor Server Bootstrap

**Entities**

- Runtime Configuration
  - Fields: `PORT` (u16), `HEALTH_PATH` (string), `LOG_LEVEL` (enum/string)
  - Rules: env > .env > defaults; validate `PORT` in 1..=65535; `HEALTH_PATH` non-empty, leading `/`.

- Health Probe
  - Fields: `status` (string: "ok"), optional `timestamp` (ISO8601), optional `version` (string)
  - Rules: must serialize to JSON; no sensitive data.

**Relationships**

- Server startup depends on valid Runtime Configuration.
- Health Probe is served by the server using current configuration.

**State Transitions**

- Startup → Running (on successful bind and configuration load)
- Running → ErrorExit (on bind failure or invalid configuration)
- Running → GracefulShutdown (on SIGINT/SIGTERM, bounded time)