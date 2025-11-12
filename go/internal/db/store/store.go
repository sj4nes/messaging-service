package store

import (
    "context"

    "github.com/jackc/pgx/v5/pgxpool"
    "github.com/sj4nes/messaging-service/go/api"
    "github.com/sj4nes/messaging-service/go/api/models"
    "github.com/sj4nes/messaging-service/go/internal/db/repository"
)

// Store implements api.StoreInterface backed by PostgreSQL repositories.
type Store struct {
    conv *repository.ConversationsRepository
    // msgs *repository.MessagesRepository // to be added with T044
}

func New(pool *pgxpool.Pool) *Store {
    return &Store{
        conv: repository.NewConversationsRepository(pool),
    }
}

func (s *Store) ListConversations(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error) {
    return s.conv.List(ctx, page, size)
}

func (s *Store) ListMessages(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error) {
    // TODO: implement once MessagesRepository is added (T044)
    return []models.MessageDto{}, 0, nil
}

// Ensure Store satisfies the interface at compile time
var _ api.StoreInterface = (*Store)(nil)
 
