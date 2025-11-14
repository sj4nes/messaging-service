package worker

import (
	"context"
	"log"
	"time"

	"github.com/sj4nes/messaging-service/go/internal/queue"
)

// Handler processes a single outbound message event.
type Handler func(ctx context.Context, evt queue.OutboundMessageEvent) error

// Worker consumes events from the queue and invokes the handler.
type Worker struct {
	q        queue.Queue
	h        Handler
	pollWait time.Duration
}

// New returns a worker that consumes from q and processes via h.
func New(q queue.Queue, h Handler) *Worker {
	return &Worker{q: q, h: h, pollWait: 50 * time.Millisecond}
}

// Start runs the processing loop until the context is canceled.
func (w *Worker) Start(ctx context.Context) {
	for {
		select {
		case <-ctx.Done():
			return
		default:
		}
		deliv, err := w.q.Receive(ctx)
		if err != nil {
			// Avoid hot loop on context cancellation, otherwise log and backoff lightly
			if ctx.Err() != nil {
				return
			}
			log.Printf("worker receive error: %v", err)
			time.Sleep(w.pollWait)
			continue
		}
		if err := w.h(ctx, deliv.Event); err != nil {
			// For in-memory, nack is a no-op; downstream reliable queues could requeue
			deliv.Nack(err)
			log.Printf("worker handler error: %v", err)
			continue
		}
		deliv.Ack()
	}
}
