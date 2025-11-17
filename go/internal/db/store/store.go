package store

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/sj4nes/messaging-service/go/api"
	"github.com/sj4nes/messaging-service/go/api/models"
	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	"github.com/sj4nes/messaging-service/go/internal/db/seed"
	"github.com/sj4nes/messaging-service/go/internal/queue"
)

// Store implements api.StoreInterface backed by PostgreSQL repositories.
type Store struct {
	conv *repository.ConversationsRepository
	msgs *repository.MessagesRepository // added with T044
	q    queue.Queue
	pool *pgxpool.Pool
}

func New(pool *pgxpool.Pool) *Store {
	return &Store{
		conv: repository.NewConversationsRepository(pool),
		msgs: repository.NewMessagesRepository(pool),
		pool: pool,
	}
}

// NewWithQueue configures a queue for enqueueing outbound events (US1).
func NewWithQueue(pool *pgxpool.Pool, q queue.Queue) *Store {
	s := New(pool)
	s.q = q
	return s
}

func (s *Store) ListConversations(ctx context.Context, page, size int) ([]models.ConversationDto, uint64, error) {
	// Seed a minimum baseline set of conversations/messages if the DB is empty.
	// This matches the in-memory baseline used by the in-memory store so tests
	// that rely on seeded data behave consistently across persistence modes.
	seed.SeedMinimumIfNeeded(ctx, s.pool)
	return s.conv.List(ctx, page, size)
}

func (s *Store) ListMessages(ctx context.Context, conversationID string, page, size int) ([]models.MessageDto, uint64, error) {
	// Ensure seeded baseline conversation exists when listing messages from DB.
	seed.SeedMinimumIfNeeded(ctx, s.pool)
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
	// Persist outbound message to DB via repository (best effort; return error on failure)
	if s.msgs != nil {
		id, err := s.msgs.InsertOutbound(ctx, string(typ), req.From, req.To, req.Body, req.Timestamp)
		if err != nil {
			return fmt.Errorf("failed to persist message: %w", err)
		}
		if evt.Metadata == nil {
			evt.Metadata = map[string]any{}
		}
		evt.Metadata["message_id"] = id
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
	if s.msgs != nil {
		id, err := s.msgs.InsertOutbound(ctx, string(queue.ChannelEmail), req.From, req.To, req.Body, req.Timestamp)
		if err != nil {
			return fmt.Errorf("failed to persist message: %w", err)
		}
		if evt.Metadata == nil {
			evt.Metadata = map[string]any{}
		}
		evt.Metadata["message_id"] = id
	}
	_, err := s.q.Publish(ctx, evt)
	return err
}

// CreateInboundSmsEvent persists an inbound SMS/MMS webhook event to the inbound_events table.
func (s *Store) CreateInboundSmsEvent(ctx context.Context, req *models.SmsRequest) error {
	if s.pool == nil {
		return errors.New("pool not configured")
	}

	channel := "sms"
	if strings.ToLower(req.Type) == "mms" {
		channel = "mms"
	}

	// Build payload as jsonb
	payload, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("failed to marshal payload: %w", err)
	}

	// Insert into inbound_events with idempotency on provider_message_id
	providerMsgID := req.MessagingProviderID // from SmsRequest (Twilio message SID, etc.)

	_, err = s.pool.Exec(ctx, `
		INSERT INTO inbound_events (event_type, payload, available_at, status, channel, "from", "to", provider_message_id)
		VALUES ('sms_received', $1, now(), 'pending', $2, $3, $4, $5)
		ON CONFLICT (channel, provider_message_id) WHERE provider_message_id IS NOT NULL DO NOTHING
	`, payload, channel, req.From, req.To, sql.NullString{String: providerMsgID, Valid: providerMsgID != ""})

	if err != nil {
		return fmt.Errorf("failed to insert inbound sms event: %w", err)
	}
	return nil
}

// CreateInboundEmailEvent persists an inbound Email webhook event to the inbound_events table.
func (s *Store) CreateInboundEmailEvent(ctx context.Context, req *models.EmailRequest) error {
	if s.pool == nil {
		return errors.New("pool not configured")
	}

	// Build payload as jsonb
	payload, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("failed to marshal payload: %w", err)
	}

	// Insert into inbound_events with idempotency on provider_message_id
	providerMsgID := req.XillioID

	_, err = s.pool.Exec(ctx, `
		INSERT INTO inbound_events (event_type, payload, available_at, status, channel, "from", "to", provider_message_id)
		VALUES ('email_received', $1, now(), 'pending', 'email', $2, $3, $4)
		ON CONFLICT (channel, provider_message_id) WHERE provider_message_id IS NOT NULL DO NOTHING
	`, payload, req.From, req.To, sql.NullString{String: providerMsgID, Valid: providerMsgID != ""})

	if err != nil {
		return fmt.Errorf("failed to insert inbound email event: %w", err)
	}
	return nil
}

// Ensure Store satisfies the interface at compile time
var _ api.StoreInterface = (*Store)(nil)
