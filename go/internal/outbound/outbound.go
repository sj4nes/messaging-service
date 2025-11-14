package outbound

import (
	"context"
	"errors"
	"log"
	"github.com/sony/gobreaker"

	"github.com/sj4nes/messaging-service/go/internal/metrics"
	"github.com/sj4nes/messaging-service/go/internal/providers"

	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	"github.com/sj4nes/messaging-service/go/internal/queue"
	"github.com/sj4nes/messaging-service/go/internal/state"
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
			if m != nil {
				m.RecordInvalidRouting()
			}
			log.Printf("invalid channel for provider routing: %v", evt.Channel)
			return nil
		}

		provider, ok := pr.Get(ch)
		if !ok {
			if m != nil {
				m.RecordInvalidRouting()
			}
			log.Printf("no provider registered for channel: %s", ch)
			return nil
		}

		// Check per-provider breaker if available; record an attempt metric
		if m != nil {
			m.RecordProviderAttempt(provider.Name())
		}

		// Build dispatch message
		dmsg := providers.OutboundMessage{Channel: ch, To: evt.To, From: evt.From, Body: evt.Body}

		// Short-circuit when provider breaker is open (per-provider breaker) or execute under breaker
		var res providers.DispatchResult
		var dispatchErr error
		if br, ok := breakers.Get(provider.Name()); ok {
			// If breaker already open, short-circuit
			if br.State() == gobreaker.StateOpen {
				if m != nil { m.RecordProviderBreakerOpen(provider.Name()) }
				log.Printf("provider breaker open; short-circuiting: %s", provider.Name())
				return nil
			}

			before := br.State()
			out, err := br.DoCtx(ctx, func(ctx context.Context) (interface{}, error) {
				r := provider.Dispatch(ctx, dmsg)
				if r.Outcome == providers.OutcomeSuccess {
					return r, nil
				}
				return r, errors.New("provider dispatch failed")
			})
			if err != nil {
				dispatchErr = err
			}
			if out != nil {
				res = out.(providers.DispatchResult)
			}
			after := br.State()
			if before != after {
				if m != nil { m.RecordProviderBreakerTransition(provider.Name()) }
			}
		} else {
			// no breaker configured for provider
			res = provider.Dispatch(ctx, dmsg)
		}

		// If DoCtx returned an error without a DispatchResult, record a provider error.
		if res == (providers.DispatchResult{}) {
			if dispatchErr != nil && m != nil {
				m.RecordProviderError(provider.Name())
			}
		}

		// Record metrics by outcome
		switch res.Outcome {
		case providers.OutcomeSuccess:
			if m != nil {
				m.RecordProviderSuccess(res.ProviderName)
			}
		case providers.OutcomeRateLimited:
			if m != nil {
				m.RecordProviderRateLimited(res.ProviderName)
			}
		case providers.OutcomeError:
			if m != nil {
				m.RecordProviderError(res.ProviderName)
			}
		case providers.OutcomeTimeout:
			if m != nil {
				m.RecordProviderError(res.ProviderName)
			}
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
