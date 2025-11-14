package providers

import "context"

// ChannelKind mirrors the channel types supported for outbound messages.
type ChannelKind string

const (
    ChannelSms   ChannelKind = "sms"
    ChannelMms   ChannelKind = "mms"
    ChannelEmail ChannelKind = "email"
)

// Outcome represents the provider dispatch result.
type Outcome int

const (
    OutcomeSuccess Outcome = iota
    OutcomeRateLimited
    OutcomeError
    OutcomeTimeout
)

// OutboundMessage contains the minimal fields used by providers.
type OutboundMessage struct {
    Channel ChannelKind
    To      string
    From    string
    Body    string
    // Attachments omitted for now
    IdempotencyKey *string
}

// DispatchResult indicates a provider's dispatch outcome.
type DispatchResult struct {
    ProviderName string
    Outcome      Outcome
}

// Provider represents a channel-specific provider implementation.
type Provider interface {
    Name() string
    Dispatch(ctx context.Context, msg OutboundMessage) DispatchResult
}

// ProviderRegistry maps ChannelKind to a Provider.
type ProviderRegistry struct {
    m map[ChannelKind]Provider
}

func NewRegistry() *ProviderRegistry {
    return &ProviderRegistry{m: map[ChannelKind]Provider{}}
}

func (pr *ProviderRegistry) Insert(channel ChannelKind, p Provider) {
    pr.m[channel] = p
}

func (pr *ProviderRegistry) Get(channel ChannelKind) (Provider, bool) {
    p, ok := pr.m[channel]
    return p, ok
}
