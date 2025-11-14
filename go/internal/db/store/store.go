package store

import (
	"context"
	"os"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/sj4nes/messaging-service/go/api"
	"github.com/sj4nes/messaging-service/go/api/models"
	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	"github.com/sj4nes/messaging-service/go/internal/db/seed"
)

// Store implements api.StoreInterface backed by PostgreSQL repositories.
type Store struct {
	pool            *pgxpool.Pool
	conv            *repository.ConversationsRepository
	msgs            *repository.MessagesRepository
	fallbackEnabled bool
	mem             api.StoreInterface // in-memory fallback
}

func New(pool *pgxpool.Pool) *Store {
	fb := false
	if v := os.Getenv("INMEMORY_FALLBACK"); v != "" {
		if v == "1" || v == "true" || v == "TRUE" {
			fb = true
		}
	}
	return &Store{
		pool:            pool,
		conv:            repository.NewConversationsRepository(pool),
		msgs:            repository.NewMessagesRepository(pool),
		fallbackEnabled: fb,
		mem:             &api.InMemoryStore{},
	}
}

// CreateSmsMessage persists an outbound SMS/MMS message using the primary store,
// falling back to in-memory behavior if configured.
func (s *Store) CreateSmsMessage(ctx context.Context, req *models.SmsRequest) error {
	// If no DB is configured, optionally delegate to in-memory behavior.
	if s.pool == nil || s.msgs == nil {
		if s.fallbackEnabled && s.mem != nil {
			return s.mem.CreateSmsMessage(ctx, req)
		}
		return nil
	}
	channel := "sms"
	if req.Type != "" {
		// Accept sms|mms, mirroring Rust validation already enforced in handler.
		channel = req.Type
	}
	_, err := s.msgs.InsertOutbound(ctx, channel, req.From, req.To, req.Body, req.Timestamp)
	return err
}

// CreateEmailMessage persists an outbound Email message using the primary store,
// falling back to in-memory behavior if configured.
func (s *Store) CreateEmailMessage(ctx context.Context, req *models.EmailRequest) error {
	if s.pool == nil || s.msgs == nil {
		if s.fallbackEnabled && s.mem != nil {
			return s.mem.CreateEmailMessage(ctx, req)
		}
		return nil
	}
	_, err := s.msgs.InsertOutbound(ctx, "email", req.From, req.To, req.Body, req.Timestamp)
	return err
}

func (s *Store) ListConversations(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error) {
	seed.SeedMinimumIfNeeded(ctx, s.pool)
	items, total, err := s.conv.List(ctx, page, size)
	if (err != nil || len(items) == 0) && s.fallbackEnabled {
		// Fallback when DB empty or errored (e.g., migrations not applied yet)
		fbItems, fbTotal, fbErr := s.mem.ListConversations(ctx, page, size)
		// Always return slice (not nil) for JSON encoding stability
		if fbItems == nil {
			fbItems = []models.ConversationDto{}
		}
		return fbItems, fbTotal, fbErr
	}
	if items == nil { // normalize nil slice
		items = []models.ConversationDto{}
	}
	return items, total, err
}

func (s *Store) ListMessages(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error) {
	seed.SeedMinimumIfNeeded(ctx, s.pool)
	msgs, total, err := s.msgs.ListByConversation(ctx, conversationID, page, size)
	// Legacy fallback: if requesting id "1" and no DB messages, attempt first conversation's messages
	if (err != nil || len(msgs) == 0) && conversationID == "1" {
		convs, _, _ := s.conv.List(ctx, 1, 1)
		if len(convs) > 0 && convs[0].ID != "1" {
			msgs2, total2, _ := s.msgs.ListByConversation(ctx, convs[0].ID, page, size)
			if len(msgs2) > 0 {
				msgs, total, err = msgs2, total2, nil
			}
		}
	}
	if (err != nil || len(msgs) == 0) && s.fallbackEnabled {
		fbMsgs, fbTotal, fbErr := s.mem.ListMessages(ctx, conversationID, page, size)
		if fbMsgs == nil {
			fbMsgs = []models.MessageDto{}
		}
		return fbMsgs, fbTotal, fbErr
	}
	if msgs == nil {
		msgs = []models.MessageDto{}
	}
	return msgs, total, err
}

// Ensure Store satisfies the interface at compile time
var _ api.StoreInterface = (*Store)(nil)
