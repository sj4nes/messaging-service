
package providers

import (
    "testing"

    "github.com/sj4nes/messaging-service/go/internal/config"
)

func TestPickOutcome_DeterministicRateLimit(t *testing.T) {
    cfg := &config.Config{ProviderSeed: 12345, ProviderRatelimitPct: 100}
    a := PickOutcomeForProvider("sms-mms", cfg)
    b := PickOutcomeForProvider("sms-mms", cfg)
    if a != b {
        t.Fatalf("expected deterministic outcomes got %v and %v", a, b)
    }
    if a != OutcomeRateLimited {
        t.Fatalf("expected rate-limited outcome for 100%% config; got %v", a)
    }
}

func TestPickOutcome_ProviderSpecificSeed(t *testing.T) {
    cfg := &config.Config{ProviderSmsSeed: 99, ProviderRatelimitPct: 100}
    a := PickOutcomeForProvider("sms-mms", cfg)
    if a != OutcomeRateLimited {
        t.Fatalf("expected rate-limited outcome for provider-specific seed; got %v", a)
    }
}
