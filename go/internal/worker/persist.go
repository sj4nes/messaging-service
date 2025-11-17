package worker

import (
	"context"
	"fmt"
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
		id, err := repo.InsertOutbound(ctx, ch, from, to, body, ts)
		if err != nil {
			// Log to help tests show worker errors
			fmt.Printf("PersistHandler: InsertOutbound failed channel=%s from=%s to=%s err=%v\n", ch, from, to, err)
			return err
		}
		// Log success (non-sensitive) for debugging
		fmt.Printf("PersistHandler: InsertOutbound success id=%s channel=%s\n", id, ch)
		return nil
	}
}
