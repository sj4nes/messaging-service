package api

import (
	"context"

	"github.com/sj4nes/messaging-service/go/api/models"
)

// StoreInterface defines minimal data access used by API handlers.
type StoreInterface interface {
	ListConversations(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error)
	ListMessages(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error)
	CreateSmsMessage(ctx context.Context, req *models.SmsRequest) error
	CreateEmailMessage(ctx context.Context, req *models.EmailRequest) error
}

// Store is the global store used by handlers. Defaults to in-memory.
var Store StoreInterface = &InMemoryStore{}

// SetStore allows main to override the store implementation.
func SetStore(s StoreInterface) { Store = s }
