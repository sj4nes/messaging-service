package mock

import (
	"context"
	"time"

	"github.com/sj4nes/messaging-service/go/internal/providers"
	"github.com/sj4nes/messaging-service/go/internal/config"
)

type EmailProvider struct{ cfg *config.Config }

func NewEmailProvider(cfg *config.Config) *EmailProvider { return &EmailProvider{cfg: cfg} }

func (e *EmailProvider) Name() string { return "email" }

func (e *EmailProvider) Dispatch(ctx context.Context, msg providers.OutboundMessage) providers.DispatchResult {
	select {
	case <-ctx.Done():
		return providers.DispatchResult{ProviderName: e.Name(), Outcome: providers.OutcomeTimeout}
	case <-time.After(10 * time.Millisecond):
	}
	outcome := providers.PickOutcomeForProvider(e.Name(), e.cfg)
	return providers.DispatchResult{ProviderName: e.Name(), Outcome: outcome}
}

var _ providers.Provider = (*EmailProvider)(nil)
