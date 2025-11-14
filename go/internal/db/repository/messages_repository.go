package repository

import (
	"context"
	"errors"
	"fmt"
	"strconv"
	"time"

	"github.com/jackc/pgx/v5/pgtype"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/sj4nes/messaging-service/go/api/models"
	"github.com/sj4nes/messaging-service/go/internal/db/generated"
	dbutil "github.com/sj4nes/messaging-service/go/internal/db/util"
)

// MessagesRepository provides DB access for messages scoped to a conversation.
type MessagesRepository struct {
	pool *pgxpool.Pool
	q    *generated.Queries
}

func NewMessagesRepository(pool *pgxpool.Pool) *MessagesRepository {
	var q *generated.Queries
	if pool != nil {
		q = generated.New(pool)
	}
	return &MessagesRepository{pool: pool, q: q}
}

// InsertOutbound persists an outbound message and ensures a durable conversation exists.
// It mirrors the Rust insert_outbound behavior: body dedupe, conversation upsert, and idempotency
// on (conversation, direction='outbound', sent_at, body_id).
func (r *MessagesRepository) InsertOutbound(ctx context.Context, channel, from, to, body, timestamp string) (string, error) {
	if r.pool == nil || r.q == nil {
		return "", errors.New("pool nil")
	}
	// Parse timestamp; fall back to now on error, matching Rust logic.
	var ts time.Time
	if t, err := time.Parse(time.RFC3339, timestamp); err == nil {
		ts = t
	} else {
		ts = time.Now().UTC()
	}
	params := generated.InsertOutboundMessageParams{
		Channel:        pgtype.Text{String: channel, Valid: true},
		ParticipantA:   pgtype.Text{String: from, Valid: true},
		ParticipantB:   pgtype.Text{String: to, Valid: true},
		Body:           body,
		LastActivityAt: pgtype.Timestamptz{Time: ts, Valid: true},
	}
	id, err := r.q.InsertOutboundMessage(ctx, params)
	if err != nil {
		return "", fmt.Errorf("insert outbound message: %w", err)
	}
	return id, nil
}

// ListByConversation returns paged messages for a conversation in ascending timestamp order.
// Body and Snippet are placeholders until body retrieval is implemented (future task).
func (r *MessagesRepository) ListByConversation(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error) {
	if r.pool == nil || r.q == nil {
		return nil, 0, errors.New("pool nil")
	}
	if page <= 0 {
		page = 1
	}
	// IDs are numeric in schema; attempt parse.
	convID, err := strconv.ParseInt(conversationID, 10, 64)
	if err != nil {
		// Non-numeric IDs unsupported in DB mode; return empty without error for parity.
		return []models.MessageDto{}, 0, nil
	}
	// total count
	var total uint64
	if err := r.pool.QueryRow(ctx, `SELECT COUNT(*) FROM messages WHERE conversation_id=$1`, convID).Scan(&total); err != nil {
		return nil, 0, fmt.Errorf("count messages: %w", err)
	}
	if size <= 0 {
		size = 50
	}
	if page <= 0 {
		page = 1
	}
	offset := int32((page - 1) * size)
	limit := int32(size)
	rows, err := r.q.ListMessagesForConversation(ctx, generated.ListMessagesForConversationParams{ID: convID, Limit: limit, Offset: offset})
	if err != nil {
		return nil, 0, fmt.Errorf("query messages: %w", err)
	}
	msgs := make([]models.MessageDto, 0, len(rows))
	for _, row := range rows {
		m := models.MessageDto{
			ID:        row.MID,
			Direction: row.Direction,
			Channel:   dbutil.TextToString(row.Channel),
			From:      dbutil.TextToString(row.FromParticipant),
			To:        dbutil.TextToString(row.ToParticipant),
			Body:      "", // body retrieval not yet implemented
			Snippet:   "", // snippet derivation not yet implemented
			Timestamp: dbutil.TimeToRFC3339(row.Timestamp),
		}
		msgs = append(msgs, m)
	}
	return msgs, total, nil
}

// SetOutboundProvider sets the provider_id for a given message by provider name.
// Best-effort: if the provider name can't be resolved, the update is a no-op and returns false.
func (r *MessagesRepository) SetOutboundProvider(ctx context.Context, messageID string, providerName string) (bool, error) {
	if r.pool == nil || r.q == nil {
		return false, errors.New("pool nil")
	}
	// Find provider id by name
	var pid int64
	if err := r.pool.QueryRow(ctx, `SELECT id FROM providers WHERE name=$1 LIMIT 1`, providerName).Scan(&pid); err != nil {
		return false, err
	}
	// Update messages with provider id
	if _, err := r.pool.Exec(ctx, `UPDATE messages SET provider_id = $1 WHERE id::text = $2`, pid, messageID); err != nil {
		return false, err
	}
	return true, nil
}

// Note: body and snippet fields are placeholders until message
// content storage and redaction policy are finalized.
