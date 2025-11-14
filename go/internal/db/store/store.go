package store

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/sj4nes/messaging-service/go/api"
	"github.com/sj4nes/messaging-service/go/api/models"
	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	"github.com/sj4nes/messaging-service/go/internal/queue"
)

// Store implements api.StoreInterface backed by PostgreSQL repositories.
type Store struct {
	conv *repository.ConversationsRepository
	msgs *repository.MessagesRepository // added with T044
	q    queue.Queue
}

func New(pool *pgxpool.Pool) *Store {
	return &Store{
		conv: repository.NewConversationsRepository(pool),
		msgs: repository.NewMessagesRepository(pool),
	}
}

// NewWithQueue configures a queue for enqueueing outbound events (US1).
func NewWithQueue(pool *pgxpool.Pool, q queue.Queue) *Store {
	s := New(pool)
	s.q = q
	return s
}

func (s *Store) ListConversations(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error) {
	return s.conv.List(ctx, page, size)
}

func (s *Store) ListMessages(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error) {
	return s.msgs.ListByConversation(ctx, conversationID, page, size)
}

// CreateSmsMessage enqueues an outbound message event for SMS/MMS.
func (s *Store) CreateSmsMessage(ctx context.Context, req *models.SmsRequest) error {
	if s.q == nil {
		return errors.New("queue not configured")
	}
	typ := queue.ChannelSMS
	if req.Type == "mms" {
		typ = queue.ChannelSMS // same channel; attachments are part of body/metadata for now
	}
	evt := queue.OutboundMessageEvent{
		SchemaVersion: 1,
		Channel:       typ,
		CustomerID:    "", // TODO: fill when available in request/auth context
		From:          req.From,
		To:            req.To,
		Body:          req.Body,
		Metadata:      map[string]any{"attachments": req.Attachments},
	}
	if ts := strings.TrimSpace(req.Timestamp); ts != "" {
		if t, err := time.Parse(time.RFC3339, ts); err == nil {
			evt.SentAt = &t
		}
	}
	_, err := s.q.Publish(ctx, evt)
	return err
}

// CreateEmailMessage enqueues an outbound message event for Email.
func (s *Store) CreateEmailMessage(ctx context.Context, req *models.EmailRequest) error {
	if s.q == nil {
		return errors.New("queue not configured")
	}
	evt := queue.OutboundMessageEvent{
		SchemaVersion: 1,
		Channel:       queue.ChannelEmail,
		CustomerID:    "", // TODO: fill when available in request/auth context
		From:          req.From,
		To:            req.To,
		Body:          req.Body,
		Metadata:      map[string]any{"attachments": req.Attachments},
	}
	if ts := strings.TrimSpace(req.Timestamp); ts != "" {
		if t, err := time.Parse(time.RFC3339, ts); err == nil {
			evt.SentAt = &t
		}
	}
	_, err := s.q.Publish(ctx, evt)
	return err
}

// Ensure Store satisfies the interface at compile time
var _ api.StoreInterface = (*Store)(nil)
