package mock

import (
    "context"
    "time"

    "github.com/sj4nes/messaging-service/go/internal/providers"
)

type SmsMmsProvider struct{}

func NewSmsMmsProvider() *SmsMmsProvider { return &SmsMmsProvider{} }

func (s *SmsMmsProvider) Name() string { return "sms-mms" }

func (s *SmsMmsProvider) Dispatch(ctx context.Context, msg providers.OutboundMessage) providers.DispatchResult {
    // Simulate a small delay
    select {
    case <-ctx.Done():
        return providers.DispatchResult{ProviderName: s.Name(), Outcome: providers.OutcomeTimeout}
    case <-time.After(5 * time.Millisecond):
    }
    // Very simple deterministic behavior for now: succeed
    return providers.DispatchResult{ProviderName: s.Name(), Outcome: providers.OutcomeSuccess}
}

// Implement Provider interface compatibility
var _ providers.Provider = (*SmsMmsProvider)(nil)
