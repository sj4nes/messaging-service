package repository

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

// newTestPool creates a pgxpool.Pool for tests using DATABASE_URL.
// Tests are skipped if DATABASE_URL is not set to avoid accidental use of
// a production database.
func newTestPool(t *testing.T) *pgxpool.Pool {
	t.Helper()
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		t.Skip("DATABASE_URL not set; skipping DB integration tests")
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	pool, err := pgxpool.New(ctx, url)
	if err != nil {
		t.Fatalf("failed to create pgx pool: %v", err)
	}
	t.Cleanup(func() { pool.Close() })
	return pool
}

// resetTestDB truncates the core tables we care about between tests.
// This assumes migrations have already been applied.
func resetTestDB(t *testing.T, pool *pgxpool.Pool) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	// Truncate dependent tables first to satisfy FKs, then core tables.
	stmts := []string{
		"TRUNCATE TABLE messages RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE conversations RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE message_bodies RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE customers RESTART IDENTITY CASCADE",
	}
	for _, stmt := range stmts {
		if _, err := pool.Exec(ctx, stmt); err != nil {
			t.Fatalf("failed to truncate with %q: %v", stmt, err)
		}
	}
}

// TestMessagesRepository_InsertOutbound_CreatesConversationAndMessage verifies that
// InsertOutbound persists a conversation and an outbound message, wiring IDs correctly.
func TestMessagesRepository_InsertOutbound_CreatesConversationAndMessage(t *testing.T) {
	pool := newTestPool(t)
	resetTestDB(t, pool)

	// Insert a default customer to satisfy FK on conversations.customer_id.
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if _, err := pool.Exec(ctx, `INSERT INTO customers (id, external_id, display_name) VALUES (1, 'test-customer', 'Test Customer')`); err != nil {
		t.Fatalf("failed to insert test customer: %v", err)
	}

	repo := NewMessagesRepository(pool)
	ctx, cancel = context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	channel := "sms"
	from := "+15551234567"
	to := "+15557654321"
	body := "hello from go tests"
	ts := time.Now().UTC().Format(time.RFC3339)

	msgID, err := repo.InsertOutbound(ctx, channel, from, to, body, ts)
	if err != nil {
		t.Fatalf("InsertOutbound returned error: %v", err)
	}
	if msgID == "" {
		t.Fatalf("expected non-empty message ID")
	}

	// Verify that the message row exists and is outbound and linked to a conversation.
	var (
		storedID        string
		storedDirection string
		storedChannel   string
		storedFrom      string
		storedTo        string
		storedConvID    int64
		storedBodyID    int64
	)
	row := pool.QueryRow(ctx, `
		SELECT m.id, m.direction, conv.channel, conv.participant_a, conv.participant_b, m.conversation_id, m.body_id
		FROM messages m
		JOIN conversations conv ON m.conversation_id = conv.id
		WHERE m.id = $1
	`, msgID)
	if err := row.Scan(&storedID, &storedDirection, &storedChannel, &storedFrom, &storedTo, &storedConvID, &storedBodyID); err != nil {
		t.Fatalf("failed to query stored message: %v", err)
	}
	if storedDirection != "outbound" {
		t.Errorf("expected direction 'outbound', got %q", storedDirection)
	}
	if storedChannel != channel {
		t.Errorf("expected channel %q, got %q", channel, storedChannel)
	}
	if storedFrom != from || storedTo != to {
		t.Errorf("expected from/to %q->%q, got %q->%q", from, to, storedFrom, storedTo)
	}
	if storedConvID == 0 {
		t.Errorf("expected non-zero conversation id")
	}
	if storedBodyID == 0 {
		t.Errorf("expected non-zero body id")
	}
}

// TestMessagesRepository_InsertOutbound_Idempotent ensures that two InsertOutbound
// calls with the same logical message parameters yield the same message row.
func TestMessagesRepository_InsertOutbound_Idempotent(t *testing.T) {
	pool := newTestPool(t)
	resetTestDB(t, pool)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if _, err := pool.Exec(ctx, `INSERT INTO customers (id, external_id, display_name) VALUES (1, 'test-customer', 'Test Customer')`); err != nil {
		t.Fatalf("failed to insert test customer: %v", err)
	}

	repo := NewMessagesRepository(pool)
	ctx, cancel = context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	channel := "sms"
	from := "+15551234567"
	to := "+15557654321"
	body := "idempotent check"
	ts := time.Now().UTC().Format(time.RFC3339)

	firstID, err := repo.InsertOutbound(ctx, channel, from, to, body, ts)
	if err != nil {
		t.Fatalf("first InsertOutbound returned error: %v", err)
	}
	secondID, err := repo.InsertOutbound(ctx, channel, from, to, body, ts)
	if err != nil {
		t.Fatalf("second InsertOutbound returned error: %v", err)
	}
	if firstID != secondID {
		t.Fatalf("expected idempotent IDs to match, got %q and %q", firstID, secondID)
	}

	// Verify that only one message row exists for this conversation/body/timestamp combination.
	var count int
	row := pool.QueryRow(ctx, `SELECT COUNT(*) FROM messages`)
	if err := row.Scan(&count); err != nil {
		t.Fatalf("failed to count messages: %v", err)
	}
	if count != 1 {
		t.Fatalf("expected exactly 1 message row, got %d", count)
	}
}

// TestMessagesRepository_InsertOutbound_TimestampFallback verifies that an invalid
// timestamp string does not cause failure and that a row is written.
func TestMessagesRepository_InsertOutbound_TimestampFallback(t *testing.T) {
	pool := newTestPool(t)
	resetTestDB(t, pool)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if _, err := pool.Exec(ctx, `INSERT INTO customers (id, external_id, display_name) VALUES (1, 'test-customer', 'Test Customer')`); err != nil {
		t.Fatalf("failed to insert test customer: %v", err)
	}

	repo := NewMessagesRepository(pool)
	ctx, cancel = context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	channel := "sms"
	from := "+15551234567"
	to := "+15557654321"
	body := "bad timestamp"
	badTS := "not-a-timestamp"

	msgID, err := repo.InsertOutbound(ctx, channel, from, to, body, badTS)
	if err != nil {
		t.Fatalf("InsertOutbound returned error with bad timestamp: %v", err)
	}
	if msgID == "" {
		t.Fatalf("expected non-empty message ID")
	}

	// We don't assert the exact stored timestamp, only that a row exists.
	var count int
	row := pool.QueryRow(ctx, `SELECT COUNT(*) FROM messages WHERE id = $1`, msgID)
	if err := row.Scan(&count); err != nil {
		t.Fatalf("failed to count messages by id: %v", err)
	}
	if count != 1 {
		t.Fatalf("expected 1 message row for id %s, got %d", msgID, count)
	}
}
