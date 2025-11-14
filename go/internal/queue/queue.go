package queue

import (
	"context"
	"time"
)

// Channel represents the outbound channel type.
type Channel string

const (
	ChannelSMS   Channel = "sms"
	ChannelEmail Channel = "email"
)

// OutboundMessageEvent is the normalized event enqueued by HTTP handlers.
type OutboundMessageEvent struct {
	SchemaVersion  int                    `json:"schema_version"`
	Channel        Channel                `json:"channel"`
	CustomerID     string                 `json:"customer_id"`
	From           string                 `json:"from"`
	To             string                 `json:"to"`
	Subject        *string                `json:"subject,omitempty"`
	Body           string                 `json:"body"`
	BodyHash       *string                `json:"body_hash,omitempty"`
	SentAt         *time.Time             `json:"sent_at,omitempty"`
	IdempotencyKey *string                `json:"idempotency_key,omitempty"`
	Metadata       map[string]any         `json:"metadata,omitempty"`
}

// Delivery wraps an event with ack/nack signaling for the queue implementation.
type Delivery struct {
	Event OutboundMessageEvent
	ack   func()
	nack  func(error)
}

// Ack marks the delivery as successfully processed.
func (d Delivery) Ack() { if d.ack != nil { d.ack() } }

// Nack marks the delivery as failed and eligible for retry.
func (d Delivery) Nack(err error) { if d.nack != nil { d.nack(err) } }

// NewDelivery constructs a Delivery with provided ack/nack callbacks.
func NewDelivery(evt OutboundMessageEvent, ack func(), nack func(error)) Delivery {
	return Delivery{Event: evt, ack: ack, nack: nack}
}

// Queue defines the minimal operations for an input-events queue.
type Queue interface {
	// Publish enqueues an event. It may compute and return an idempotency key.
	Publish(ctx context.Context, evt OutboundMessageEvent) (string, error)
	// Receive blocks until a delivery is available or the context is canceled.
	Receive(ctx context.Context) (Delivery, error)
	// Depth returns a best-effort gauge of pending events.
	Depth() int
}
