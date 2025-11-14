package outbound

import (
    "context"
    "log"
    

    "github.com/sj4nes/messaging-service/go/internal/metrics"
    "github.com/sj4nes/messaging-service/go/internal/providers"
    
    "github.com/sj4nes/messaging-service/go/internal/state"
    "github.com/sj4nes/messaging-service/go/internal/queue"
    "github.com/sj4nes/messaging-service/go/internal/db/repository"
)

// DispatchHandler returns a function that will be used by worker to handle outbound events
// by routing to configured provider registry and executing provider dispatch.
func DispatchHandler(pr *providers.ProviderRegistry, breakers *state.ProviderBreakers, repo *repository.MessagesRepository, m *metrics.Registry) func(ctx context.Context, evt queue.OutboundMessageEvent) error {
    return func(ctx context.Context, evt queue.OutboundMessageEvent) error {
        // Map queue channel to provider channel
        var ch providers.ChannelKind
        switch evt.Channel {
        case queue.ChannelSMS:
            // preserve MMS distinction via metadata if needed
            ch = providers.ChannelSms
        case queue.ChannelEmail:
            ch = providers.ChannelEmail
        default:
            if m != nil { m.RecordInvalidRouting() }
            log.Printf("invalid channel for provider routing: %v", evt.Channel)
            return nil
        }

        provider, ok := pr.Get(ch)
        if !ok {
            if m != nil { m.RecordInvalidRouting() }
            log.Printf("no provider registered for channel: %s", ch)
            return nil
        }

        // Check per-provider breaker if available
        // For simple parity, just attempt dispatch — we do not short-circuit here because
        // the resilience Breaker class needs a wrapper. We'll record provider attempt now.
        if m != nil { m.RecordProviderAttempt(provider.Name()) }

        // Execute provider dispatch (no heavy retries here; provider returns outcome)
        dmsg := providers.OutboundMessage{
            Channel: ch,
            To:      evt.To,
            From:    evt.From,
            Body:    evt.Body,
        }
        // if we have a message id present, we can store provider mapping later
        res := provider.Dispatch(ctx, dmsg)

        // Record metrics by outcome
        switch res.Outcome {
        case providers.OutcomeSuccess:
            if m != nil { m.RecordProviderSuccess(res.ProviderName) }
        case providers.OutcomeRateLimited:
            if m != nil { m.RecordProviderRateLimited(res.ProviderName) }
        case providers.OutcomeError:
            if m != nil { m.RecordProviderError(res.ProviderName) }
        case providers.OutcomeTimeout:
            if m != nil { m.RecordProviderError(res.ProviderName) }
        }

        // Optional: set provider on persisted message if metadata contains message_id
        if evt.Metadata != nil {
            if mid, ok := evt.Metadata["message_id"].(string); ok && mid != "" {
                if _, err := repo.SetOutboundProvider(ctx, mid, res.ProviderName); err != nil {
                    log.Printf("failed to set outbound provider for message %s: %v", mid, err)
                }
            }
        }
        return nil
    }
}
 
