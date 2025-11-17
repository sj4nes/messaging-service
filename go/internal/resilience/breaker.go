package resilience

import (
	"context"
	"time"

	"github.com/sony/gobreaker"
)

// Breaker wraps gobreaker.CircuitBreaker with simplified Execute helpers.
type Breaker struct {
	cb *gobreaker.CircuitBreaker
}

func New(name string) *Breaker {
	st := gobreaker.Settings{Name: name}
	return &Breaker{cb: gobreaker.NewCircuitBreaker(st)}
}

func (b *Breaker) Do(fn func() (interface{}, error)) (interface{}, error) {
	return b.cb.Execute(fn)
}

// DoCtx executes with context cancellation awareness.
func (b *Breaker) DoCtx(ctx context.Context, fn func(context.Context) (interface{}, error)) (interface{}, error) {
	return b.cb.Execute(func() (interface{}, error) {
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}
		return fn(ctx)
	})
}

// State returns current breaker state.
func (b *Breaker) State() gobreaker.State { return b.cb.State() }

// HalfOpenFor indicates remaining duration before evaluating half-open state (approx).
func (b *Breaker) HalfOpenFor() time.Duration { return time.Second * 0 } // placeholder
