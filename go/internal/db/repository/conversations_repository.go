package repository

import (
    "context"
    "errors"
    "fmt"

    "github.com/jackc/pgx/v5/pgxpool"
    "github.com/sj4nes/messaging-service/go/api/models"
)

// ConversationsRepository provides DB access for conversations.
type ConversationsRepository struct {
    pool *pgxpool.Pool
}

func NewConversationsRepository(pool *pgxpool.Pool) *ConversationsRepository {
    return &ConversationsRepository{pool: pool}
}

// List returns a page of conversations ordered by last activity desc.
func (r *ConversationsRepository) List(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error) {
    if r.pool == nil { return nil, 0, errors.New("pool nil") }
    if size <= 0 { size = 50 }
    if page <= 0 { page = 1 }
    offset := (page - 1) * size
    rows, err := r.pool.Query(ctx, `SELECT id, key, channel, participant_a, participant_b, message_count, last_activity_at FROM conversations ORDER BY last_activity_at DESC LIMIT $1 OFFSET $2`, size, offset)
    if err != nil { return nil, 0, fmt.Errorf("query conversations: %w", err) }
    defer rows.Close()
    var list []models.ConversationDto
    for rows.Next() {
        var c models.ConversationDto
        if err := rows.Scan(&c.ID, &c.Key, &c.Channel, &c.ParticipantA, &c.ParticipantB, &c.MessageCount, &c.LastActivity); err != nil {
            return nil, 0, fmt.Errorf("scan conversation: %w", err)
        }
        c.ID = fmt.Sprintf("%v", c.ID) // ensure string formatting if numeric
        list = append(list, c)
    }
    // total count (approx) - separate query; optimize later
    var total uint64
    if err := r.pool.QueryRow(ctx, `SELECT COUNT(*) FROM conversations`).Scan(&total); err != nil {
        return nil, 0, fmt.Errorf("count conversations: %w", err)
    }
    return list, total, nil
}
 
