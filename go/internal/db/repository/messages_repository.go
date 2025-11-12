package repository

import (
    "context"
    "errors"
    "fmt"
    "strconv"
    "time"

    "github.com/jackc/pgx/v5/pgxpool"
    "github.com/jackc/pgx/v5/pgtype"
    "github.com/sj4nes/messaging-service/go/api/models"
    "github.com/sj4nes/messaging-service/go/internal/db/generated"
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

// ListByConversation returns paged messages for a conversation in ascending timestamp order.
// Body and Snippet are placeholders until body retrieval is implemented (future task).
func (r *MessagesRepository) ListByConversation(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error) {
    if r.pool == nil || r.q == nil {
        return nil, 0, errors.New("pool nil")
    }
    if size <= 0 {
        size = 50
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
            Channel:   textOrEmpty(row.Channel),
            From:      textOrEmpty(row.FromParticipant),
            To:        textOrEmpty(row.ToParticipant),
            Body:      "",        // body retrieval not yet implemented
            Snippet:   "",        // snippet derivation not yet implemented
            Timestamp: timeOrEmpty(row.Timestamp),
        }
        msgs = append(msgs, m)
    }
    // total count
    var total uint64
    if err := r.pool.QueryRow(ctx, `SELECT COUNT(*) FROM messages WHERE conversation_id=$1`, convID).Scan(&total); err != nil {
        return nil, 0, fmt.Errorf("count messages: %w", err)
    }
    return msgs, total, nil
}

// Helpers to safely unwrap pgtype values from sqlc generated rows.
func textOrEmpty(t pgtype.Text) string {
    if !t.Valid {
        return ""
    }
    return t.String
}

func timeOrEmpty(ts pgtype.Timestamptz) string {
    if !ts.Valid {
        return ""
    }
    return ts.Time.UTC().Format(time.RFC3339)
}
