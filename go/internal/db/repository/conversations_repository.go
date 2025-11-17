package repository

import (
	"context"
	"errors"
	"fmt"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/sj4nes/messaging-service/go/api/models"
	"github.com/sj4nes/messaging-service/go/internal/db/generated"
	dbutil "github.com/sj4nes/messaging-service/go/internal/db/util"
)

// ConversationsRepository provides DB access for conversations.
type ConversationsRepository struct {
	pool *pgxpool.Pool
	q    *generated.Queries
}

func NewConversationsRepository(pool *pgxpool.Pool) *ConversationsRepository {
	var q *generated.Queries
	if pool != nil {
		q = generated.New(pool)
	}
	return &ConversationsRepository{pool: pool, q: q}
}

// List returns a page of conversations ordered by last activity desc.
func (r *ConversationsRepository) List(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error) {
	if r.pool == nil || r.q == nil {
		return nil, 0, errors.New("pool nil")
	}
	if page <= 0 {
		page = 1
	}
	// compute total first
	var total uint64
	if err := r.pool.QueryRow(ctx, `SELECT COUNT(*) FROM conversations`).Scan(&total); err != nil {
		return nil, 0, fmt.Errorf("count conversations: %w", err)
	}
	if size <= 0 {
		size = 50
	}
	if page <= 0 {
		page = 1
	}
	offset := int32((page - 1) * size)
	limit := int32(size)
	rows, err := r.q.ListConversations(ctx, generated.ListConversationsParams{Limit: limit, Offset: offset})
	if err != nil {
		return nil, 0, fmt.Errorf("query conversations: %w", err)
	}
	list := make([]models.ConversationDto, 0, len(rows))
	for _, row := range rows {
		c := models.ConversationDto{
			ID:           row.ID,
			Key:          dbutil.TextToString(row.Key),
			Channel:      dbutil.TextToString(row.Channel),
			ParticipantA: dbutil.TextToString(row.ParticipantA),
			ParticipantB: dbutil.TextToString(row.ParticipantB),
			MessageCount: uint32(row.MessageCount),
			LastActivity: dbutil.TimeToRFC3339(row.LastActivityAt),
		}
		list = append(list, c)
	}
	return list, total, nil
}
