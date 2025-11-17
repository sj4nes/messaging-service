package api

import (
	"context"

	"github.com/sj4nes/messaging-service/go/api/models"
)

// InMemoryStore provides a simple in-memory dataset for parity and tests.
type InMemoryStore struct{}

// Sample data kept here to avoid leaking into handlers.
var (
	imConversations = []models.ConversationDto{
		{ID: "c1", Key: "conv:c1", Channel: "sms", ParticipantA: "+15550001", ParticipantB: "+15550002", MessageCount: 2, LastActivity: "2025-11-10T12:00:00Z"},
		{ID: "c2", Key: "conv:c2", Channel: "email", ParticipantA: "alice@example.com", ParticipantB: "bob@example.com", MessageCount: 1, LastActivity: "2025-11-10T12:05:00Z"},
	}
	imMessages = map[string][]models.MessageDto{
		"c1": {
			{ID: "m1", Direction: "outbound", Channel: "sms", From: "+15550001", To: "+15550002", Body: "Hello there", Snippet: "Hello there", Timestamp: "2025-11-10T12:00:00Z"},
			{ID: "m2", Direction: "inbound", Channel: "sms", From: "+15550002", To: "+15550001", Body: "Hi!", Snippet: "Hi!", Timestamp: "2025-11-10T12:01:00Z"},
		},
		"1": {
			{ID: "m1", Direction: "outbound", Channel: "sms", From: "+15550001", To: "+15550002", Body: "Hello there", Snippet: "Hello there", Timestamp: "2025-11-10T12:00:00Z"},
			{ID: "m2", Direction: "inbound", Channel: "sms", From: "+15550002", To: "+15550001", Body: "Hi!", Snippet: "Hi!", Timestamp: "2025-11-10T12:01:00Z"},
		},
		"2": {
			{ID: "m3", Direction: "outbound", Channel: "email", From: "alice@example.com", To: "bob@example.com", Body: "Status update", Snippet: "Status update", Timestamp: "2025-11-10T12:05:00Z"},
		},
	}
)

func (s *InMemoryStore) ListConversations(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error) {
	total := uint64(len(imConversations))
	if size <= 0 {
		size = 50
	}
	start := (page - 1) * size
	if start > len(imConversations) {
		start = len(imConversations)
	}
	end := start + size
	if end > len(imConversations) {
		end = len(imConversations)
	}
	return imConversations[start:end], total, nil
}

func (s *InMemoryStore) ListMessages(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error) {
	msgs := imMessages[conversationID]
	if msgs == nil {
		msgs = []models.MessageDto{}
	}
	total := uint64(len(msgs))
	if size <= 0 {
		size = 50
	}
	start := (page - 1) * size
	if start > len(msgs) {
		start = len(msgs)
	}
	end := start + size
	if end > len(msgs) {
		end = len(msgs)
	}
	return msgs[start:end], total, nil
}

// CreateSmsMessage is a no-op persistence for SMS/MMS in the in-memory store.
// It accepts the request for interface compatibility but does not mutate state.
func (s *InMemoryStore) CreateSmsMessage(ctx context.Context, req *models.SmsRequest) error {
	return nil
}

// CreateEmailMessage is a no-op persistence for Email in the in-memory store.
// It accepts the request for interface compatibility but does not mutate state.
func (s *InMemoryStore) CreateEmailMessage(ctx context.Context, req *models.EmailRequest) error {
	return nil
}

// CreateInboundSmsEvent is a no-op for inbound SMS/MMS webhooks in the in-memory store.
func (s *InMemoryStore) CreateInboundSmsEvent(ctx context.Context, req *models.SmsRequest) error {
	return nil
}

// CreateInboundEmailEvent is a no-op for inbound Email webhooks in the in-memory store.
func (s *InMemoryStore) CreateInboundEmailEvent(ctx context.Context, req *models.EmailRequest) error {
	return nil
}
