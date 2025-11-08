# Observability

This document describes the observability capabilities of the messaging service, including metrics, logging, and monitoring best practices.

## Metrics Endpoint

The service exposes Prometheus-compatible metrics at `GET /metrics`.

### Response Format

JSON snapshot of all counters and gauges:

```json
{
  "ts_unix_ms": 1699123456789,
  "rate_limited": 0,
  "breaker_open": 0,
  "dispatch_attempts": 42,
  "dispatch_success": 40,
  "dispatch_rate_limited": 1,
  "dispatch_error": 1,
  "conversations_created": 15,
  "conversations_reused": 25,
  "conversations_failures": 2,
  ...
}
```

### Available Metrics

#### Conversation Metrics (Feature 009)

| Metric | Type | Description |
|--------|------|-------------|
| `conversations_created` | Counter | Total number of new conversations created |
| `conversations_reused` | Counter | Total number of times existing conversations were matched/reused |
| `conversations_failures` | Counter | Total number of conversation upsert failures |

**Usage**: Track conversation creation patterns, deduplication effectiveness, and error rates.

**Alert Recommendations**:
- **High failure rate**: `conversations_failures / (conversations_created + conversations_reused) > 0.05` (5% error rate)
- **No conversation reuse**: `conversations_reused == 0` for extended periods may indicate normalization issues

#### Dispatch Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `dispatch_attempts` | Counter | Total message dispatch attempts |
| `dispatch_success` | Counter | Successful dispatches |
| `dispatch_rate_limited` | Counter | Requests rejected due to rate limiting |
| `dispatch_error` | Counter | Failed dispatch attempts |

#### Provider Metrics (Feature 008)

Per-provider counters for SMS/MMS and Email channels:

| Metric | Type | Description |
|--------|------|-------------|
| `provider_sms_mms_attempts` | Counter | SMS/MMS provider attempts |
| `provider_sms_mms_success` | Counter | SMS/MMS successful sends |
| `provider_sms_mms_rate_limited` | Counter | SMS/MMS rate-limited requests |
| `provider_sms_mms_error` | Counter | SMS/MMS errors |
| `provider_email_attempts` | Counter | Email provider attempts |
| `provider_email_success` | Counter | Email successful sends |
| `provider_email_rate_limited` | Counter | Email rate-limited requests |
| `provider_email_error` | Counter | Email errors |

#### Circuit Breaker Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `breaker_open` | Counter | Total breaker open events (legacy) |
| `breaker_transitions` | Counter | Total breaker state transitions (legacy) |
| `provider_sms_mms_breaker_transitions` | Counter | SMS/MMS breaker transitions |
| `provider_email_breaker_transitions` | Counter | Email breaker transitions |

#### Worker Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `worker_claimed` | Counter | Messages claimed from queue |
| `worker_processed` | Counter | Messages successfully processed |
| `worker_error` | Counter | Worker processing errors |
| `worker_dead_letter` | Counter | Messages moved to dead letter queue |
| `worker_latency_avg_us` | Gauge | Average processing latency (microseconds) |
| `worker_latency_max_us` | Gauge | Maximum processing latency (microseconds) |

#### Routing Metrics (Feature 008)

| Metric | Type | Description |
|--------|------|-------------|
| `invalid_routing` | Counter | Messages with unroutable channel/address combinations |

## Logging

The service uses structured logging via the `tracing` crate with the following levels:

- **ERROR**: Critical failures requiring immediate attention
- **WARN**: Degraded performance or non-critical issues
- **INFO**: High-level operational events (server start, conversation creation)
- **DEBUG**: Detailed diagnostic information (conversation normalization, key derivation)
- **TRACE**: Very verbose debugging (SQL queries, HTTP requests)

### Key Log Events

#### Conversation Events (Feature 009)

- `conversation_created`: New conversation record inserted
  - Fields: `conversation_id`, `channel`, `participant_a`, `participant_b`, `key`
- `conversation_reused`: Existing conversation matched via unique key
  - Fields: `conversation_id`, `key`
- `conversation_upsert_failed`: Database error during conversation upsert
  - Fields: `error`, `channel`, `participants`

#### Message Events

- `message_received`: Inbound message received
- `message_dispatched`: Outbound message sent to provider
- `message_failed`: Message processing failure

## Monitoring Best Practices

### Dashboard Recommendations

1. **Conversation Health**
   - Line graph: `conversations_created` and `conversations_reused` over time
   - Gauge: Conversation reuse ratio (`conversations_reused / (conversations_created + conversations_reused)`)
   - Alert: `conversations_failures` spike detection

2. **Message Throughput**
   - Line graph: `dispatch_attempts`, `dispatch_success`, `dispatch_error` rates
   - Gauge: Error rate percentage
   - Heatmap: `worker_latency_avg_us` distribution

3. **Provider Performance**
   - Per-provider success rates: `provider_*_success / provider_*_attempts`
   - Circuit breaker states: `provider_*_breaker_transitions`
   - Rate limiting impact: `provider_*_rate_limited`

### Alert Rules

#### Critical Alerts

- **High conversation failure rate**: `conversations_failures` increase > 10 errors/min
- **Database connectivity**: `conversations_failures` combined with worker errors
- **Circuit breaker cascade**: Multiple provider breakers open simultaneously

#### Warning Alerts

- **Degraded conversation reuse**: Reuse ratio drops below expected baseline (e.g., 60%)
- **Increased latency**: `worker_latency_avg_us` > 500ms for 5 minutes
- **Provider degradation**: Any provider error rate > 5%

### Example Prometheus Queries

```promql
# Conversation creation rate (5m window)
rate(conversations_created[5m])

# Conversation reuse ratio
conversations_reused / (conversations_created + conversations_reused)

# Error rate percentage
100 * conversations_failures / (conversations_created + conversations_reused + conversations_failures)

# High latency alert
worker_latency_avg_us > 500000
```

## Tracing Integration

For distributed tracing (future enhancement):
- Add `trace_id` to all log events
- Propagate trace context via HTTP headers (W3C Trace Context)
- Export spans to OpenTelemetry collector

## Health Checks

- **Liveness**: `GET /health` (HTTP 200 = service is running)
- **Readiness**: Check database connectivity via conversation table query
- **Metrics**: `GET /metrics` (should return < 500ms)

