package memory

import (
	"context"
	"errors"
	"sync"

	"github.com/sj4nes/messaging-service/go/internal/queue"
)

var ErrClosed = errors.New("memory queue closed")

// MemoryQueue is a simple in-memory, bounded queue for development and tests.
type MemoryQueue struct {
	ch       chan queue.OutboundMessageEvent
	closed   bool
	mu       sync.RWMutex
}

// New creates a MemoryQueue with the provided capacity.
func New(capacity int) *MemoryQueue {
	if capacity <= 0 {
		capacity = 1024
	}
	return &MemoryQueue{ch: make(chan queue.OutboundMessageEvent, capacity)}
}

// Publish enqueues an event; returns idempotency key if already present or empty string.
func (m *MemoryQueue) Publish(ctx context.Context, evt queue.OutboundMessageEvent) (string, error) {
	m.mu.RLock()
	closed := m.closed
	m.mu.RUnlock()
	if closed {
		return "", ErrClosed
	}
	select {
	case <-ctx.Done():
		return "", ctx.Err()
	case m.ch <- evt:
		if evt.IdempotencyKey != nil {
			return *evt.IdempotencyKey, nil
		}
		return "", nil
	}
}

// Receive blocks for next event and wraps it in a delivery with no-op ack/nack.
func (m *MemoryQueue) Receive(ctx context.Context) (queue.Delivery, error) {
	select {
	case <-ctx.Done():
		return queue.Delivery{}, ctx.Err()
	case evt, ok := <-m.ch:
		if !ok {
			return queue.Delivery{}, ErrClosed
		}
		return queue.NewDelivery(evt, func(){}, func(error){}), nil
	}
}

// Depth returns the number of currently buffered events.
func (m *MemoryQueue) Depth() int { return len(m.ch) }

// Close closes the underlying channel; subsequent publishes will fail.
func (m *MemoryQueue) Close() {
	m.mu.Lock()
	defer m.mu.Unlock()
	if !m.closed {
		m.closed = true
		close(m.ch)
	}
}
