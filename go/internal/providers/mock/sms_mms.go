package mock

import (
	"context"
	"time"

	"github.com/sj4nes/messaging-service/go/internal/config"
	"github.com/sj4nes/messaging-service/go/internal/providers"
)

type SmsMmsProvider struct{ cfg *config.Config }

func NewSmsMmsProvider(cfg *config.Config) *SmsMmsProvider { return &SmsMmsProvider{cfg: cfg} }

func (s *SmsMmsProvider) Name() string { return "sms-mms" }

func (s *SmsMmsProvider) Dispatch(ctx context.Context, msg providers.OutboundMessage) providers.DispatchResult {
	// Simulate a small delay
	select {
	case <-ctx.Done():
		return providers.DispatchResult{ProviderName: s.Name(), Outcome: providers.OutcomeTimeout}
	case <-time.After(5 * time.Millisecond):
	}
	// Determine outcome via provider randomized behavior driven by config
	outcome := providers.PickOutcomeForProvider(s.Name(), s.cfg)
	return providers.DispatchResult{ProviderName: s.Name(), Outcome: outcome}
}

// Implement Provider interface compatibility
var _ providers.Provider = (*SmsMmsProvider)(nil)
