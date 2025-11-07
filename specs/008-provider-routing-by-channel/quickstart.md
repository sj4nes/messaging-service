# Quickstart: Provider Routing By Channel

## Configure Providers

Environment variables (optional overrides):
- API_PROVIDER_SMS_TIMEOUT_PCT
- API_PROVIDER_SMS_ERROR_PCT
- API_PROVIDER_SMS_RATELIMIT_PCT
- API_PROVIDER_SMS_SEED
- API_PROVIDER_EMAIL_TIMEOUT_PCT
- API_PROVIDER_EMAIL_ERROR_PCT
- API_PROVIDER_EMAIL_RATELIMIT_PCT
- API_PROVIDER_EMAIL_SEED

Fallback globals:
- API_PROVIDER_TIMEOUT_PCT
- API_PROVIDER_ERROR_PCT
- API_PROVIDER_RATELIMIT_PCT
- API_PROVIDER_SEED

## Run Tests

Use existing `bin/test.sh` plus new seeded tests (to be added) to verify:
1. SMS and Email route to distinct providers.
2. Breaker isolation: induce failures for sms-mms only.
3. Deterministic outcomes under fixed seeds.

## Observability
- Logs include provider_name and outcome per dispatch.
- Metrics expose provider-labeled counters.

## Failure Simulation
Adjust pct variables to create error/timeouts for a single provider; confirm other provider unaffected.
