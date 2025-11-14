package worker

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/sj4nes/messaging-service/go/internal/metrics"
	"github.com/sj4nes/messaging-service/go/internal/queue"
)

// Handler processes a single outbound message event.
type Handler func(ctx context.Context, evt queue.OutboundMessageEvent) error

// Worker consumes events from the queue and invokes the handler.
type Worker struct {
	q        queue.Queue
	h        Handler
	pollWait time.Duration
	// retry/DLQ
	mu        sync.Mutex
	attempts  map[string]int       // idempotency key -> attempts
	firstSeen map[string]time.Time // idempotency key -> first seen time
	dlq       []DeadLetter
	opts      Options
	metrics   *metrics.Registry
}

// New returns a worker that consumes from q and processes via h.
func New(q queue.Queue, h Handler) *Worker {
	return &Worker{q: q, h: h, pollWait: 50 * time.Millisecond, attempts: map[string]int{}, firstSeen: map[string]time.Time{}, opts: DefaultOptions()}
}

// NewWithOptions returns a worker with custom options and optional metrics.
func NewWithOptions(q queue.Queue, h Handler, opts Options, m *metrics.Registry) *Worker {
	if opts.MaxAttempts <= 0 {
		opts = DefaultOptions()
	}
	return &Worker{q: q, h: h, pollWait: 50 * time.Millisecond, attempts: map[string]int{}, firstSeen: map[string]time.Time{}, opts: opts, metrics: m}
}

// Options configure retry, backoff and DLQ behavior.
type Options struct {
	MaxAttempts int
	MaxAge      time.Duration
	BackoffBase time.Duration
	BackoffCap  time.Duration
}

// DefaultOptions per spec Option B: 10 attempts, 72h window, capped exponential backoff.
func DefaultOptions() Options {
	return Options{MaxAttempts: 10, MaxAge: 72 * time.Hour, BackoffBase: 200 * time.Millisecond, BackoffCap: 5 * time.Second}
}

// DeadLetter contains failed event and last error.
type DeadLetter struct {
	Event     queue.OutboundMessageEvent
	Attempts  int
	FirstSeen time.Time
	LastError string
	At        time.Time
}

// DLQLength returns the number of entries currently in the DLQ (for tests/metrics).
func (w *Worker) DLQLength() int {
	w.mu.Lock()
	defer w.mu.Unlock()
	return len(w.dlq)
}

func (w *Worker) computeKey(evt queue.OutboundMessageEvent) string {
	// channel|from|to|normalized_ts|sha256(body)
	ts := ""
	if evt.SentAt != nil {
		ts = evt.SentAt.UTC().Truncate(time.Second).Format(time.RFC3339)
	}
	bh := sha256.Sum256([]byte(evt.Body))
	base := fmt.Sprintf("%s|%s|%s|%s|%s", string(evt.Channel), evt.From, evt.To, ts, hex.EncodeToString(bh[:]))
	sum := sha256.Sum256([]byte(base))
	return hex.EncodeToString(sum[:])
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
			// Retry with backoff or DLQ per policy
			key := w.computeKey(deliv.Event)
			w.mu.Lock()
			a := w.attempts[key] + 1
			w.attempts[key] = a
			if _, ok := w.firstSeen[key]; !ok {
				w.firstSeen[key] = time.Now().UTC()
			}
			fs := w.firstSeen[key]
			w.mu.Unlock()

			if w.metrics != nil {
				w.metrics.IncRetry()
			}

			// Check limits
			if a >= w.opts.MaxAttempts || time.Since(fs) >= w.opts.MaxAge {
				w.mu.Lock()
				w.dlq = append(w.dlq, DeadLetter{Event: deliv.Event, Attempts: a, FirstSeen: fs, LastError: err.Error(), At: time.Now().UTC()})
				// cleanup attempt tracking
				delete(w.attempts, key)
				delete(w.firstSeen, key)
				w.mu.Unlock()
				if w.metrics != nil {
					w.metrics.IncDLQ()
				}
				// Ack to drop from active processing path
				deliv.Ack()
				log.Printf("worker DLQ event after %d attempts: %v", a, err)
				continue
			}

			// compute backoff
			bo := w.opts.BackoffBase * (1 << (a - 1))
			if bo > w.opts.BackoffCap {
				bo = w.opts.BackoffCap
			}
			// re-enqueue after backoff (fire-and-forget goroutine)
			go func(e queue.OutboundMessageEvent, d time.Duration) {
				timer := time.NewTimer(d)
				defer timer.Stop()
				<-timer.C
				// ignore publish error on retry path; will be observed on next receive cycle
				_, _ = w.q.Publish(context.Background(), e)
			}(deliv.Event, bo)

			// For in-memory, nack is a no-op
			deliv.Nack(err)
			log.Printf("worker handler error (attempt %d, backoff %s): %v", a, bo, err)
			continue
		}
		deliv.Ack()
	}
}
