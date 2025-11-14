package mock

import (
    "context"
    "time"

    "github.com/sj4nes/messaging-service/go/internal/providers"
)

type EmailProvider struct{}

func NewEmailProvider() *EmailProvider { return &EmailProvider{} }

func (e *EmailProvider) Name() string { return "email" }

func (e *EmailProvider) Dispatch(ctx context.Context, msg providers.OutboundMessage) providers.DispatchResult {
    select {
    case <-ctx.Done():
        return providers.DispatchResult{ProviderName: e.Name(), Outcome: providers.OutcomeTimeout}
    case <-time.After(10 * time.Millisecond):
    }
    return providers.DispatchResult{ProviderName: e.Name(), Outcome: providers.OutcomeSuccess}
}

var _ providers.Provider = (*EmailProvider)(nil)
