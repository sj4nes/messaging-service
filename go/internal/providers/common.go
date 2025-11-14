package providers

import "github.com/sj4nes/messaging-service/go/internal/config"

// PickOutcomeForProvider is a placeholder to mirror Rust deterministic mock outcomes.
// For now it returns success, but it will later consult config provider_* fields.
func PickOutcomeForProvider(name string, cfg *config.Config) Outcome {
    _ = name
    _ = cfg
    return OutcomeSuccess
}
