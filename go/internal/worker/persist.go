package worker

import (
	"context"
	"time"

	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	"github.com/sj4nes/messaging-service/go/internal/queue"
)

// PersistHandler returns a Handler that maps OutboundMessageEvent to repository.InsertOutbound.
func PersistHandler(repo *repository.MessagesRepository) Handler {
	return func(ctx context.Context, evt queue.OutboundMessageEvent) error {
		// Convert event fields
		ch := string(evt.Channel)
		from := evt.From
		to := evt.To
		body := evt.Body
		ts := ""
		if evt.SentAt != nil {
			ts = evt.SentAt.UTC().Format(time.RFC3339)
		}
		_, err := repo.InsertOutbound(ctx, ch, from, to, body, ts)
		return err
	}
}
