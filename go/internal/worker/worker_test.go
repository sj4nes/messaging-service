package worker

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"

	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	"github.com/sj4nes/messaging-service/go/internal/queue"
	qmemory "github.com/sj4nes/messaging-service/go/internal/queue/memory"
)

func newTestPool(t *testing.T) *pgxpool.Pool {
	t.Helper()
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		t.Skip("DATABASE_URL not set; skipping worker DB integration tests")
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

func resetDB(t *testing.T, pool *pgxpool.Pool) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	stmts := []string{
		"TRUNCATE TABLE messages RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE conversations RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE message_bodies RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE providers RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE customers RESTART IDENTITY CASCADE",
	}
	for _, stmt := range stmts {
		if _, err := pool.Exec(ctx, stmt); err != nil {
			t.Fatalf("failed to truncate with %q: %v", stmt, err)
		}
	}
	if _, err := pool.Exec(ctx, `INSERT INTO customers (id, name) VALUES (1, 'Test Customer')`); err != nil {
		t.Fatalf("failed to insert test customer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO providers (id, customer_id, kind, name) VALUES (1, 1, 'sms', 'Test Provider')`); err != nil {
		t.Fatalf("failed to insert test provider: %v", err)
	}
}

func waitForCount(t *testing.T, pool *pgxpool.Pool, query string, expected int, timeout time.Duration) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	deadline := time.Now().Add(timeout)
	for {
		var count int
		row := pool.QueryRow(ctx, query)
		if err := row.Scan(&count); err == nil && count == expected {
			return
		}
		if time.Now().After(deadline) {
			t.Fatalf("timeout waiting for query %q to have count %d", query, expected)
		}
		time.Sleep(20 * time.Millisecond)
	}
}

func TestWorker_PersistsOutboundEvent_CreatesMessageAndConversation(t *testing.T) {
	pool := newTestPool(t)
	resetDB(t, pool)

	mq := qmemory.New(16)
	repo := repository.NewMessagesRepository(pool)
	h := PersistHandler(repo)
	w := New(mq, h)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go w.Start(ctx)

	now := time.Now().UTC()
	evt := queue.OutboundMessageEvent{
		SchemaVersion: 1,
		Channel:       queue.ChannelSMS,
		CustomerID:    "1",
		From:          "+15551234567",
		To:            "+15557654321",
		Body:          "hello from worker",
		SentAt:        &now,
	}
	if _, err := mq.Publish(ctx, evt); err != nil {
		t.Fatalf("Publish failed: %v", err)
	}

	waitForCount(t, pool, `SELECT COUNT(*) FROM messages`, 1, 2*time.Second)
}

func TestWorker_Idempotent_DuplicateEventNoDuplicateRows(t *testing.T) {
	pool := newTestPool(t)
	resetDB(t, pool)

	mq := qmemory.New(16)
	repo := repository.NewMessagesRepository(pool)
	h := PersistHandler(repo)
	w := New(mq, h)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go w.Start(ctx)

	fixed := time.Date(2025, 11, 14, 12, 0, 0, 0, time.UTC)
	evt := queue.OutboundMessageEvent{
		SchemaVersion: 1,
		Channel:       queue.ChannelSMS,
		CustomerID:    "1",
		From:          "+15550001",
		To:            "+15550002",
		Body:          "same body",
		SentAt:        &fixed,
	}
	if _, err := mq.Publish(ctx, evt); err != nil {
		t.Fatalf("Publish failed: %v", err)
	}
	if _, err := mq.Publish(ctx, evt); err != nil {
		t.Fatalf("Publish duplicate failed: %v", err)
	}

	waitForCount(t, pool, `SELECT COUNT(*) FROM messages`, 1, 2*time.Second)
}

// TestWorker_RetryThenSuccess ensures that transient errors are retried and eventually succeed without DLQ.
func TestWorker_RetryThenSuccess(t *testing.T) {
	pool := newTestPool(t)
	resetDB(t, pool)

	mq := qmemory.New(32)
	attemptsNeeded := 3
	attemptCount := 0
	// Failing handler first 3 times, then succeed.
	h := func(ctx context.Context, evt queue.OutboundMessageEvent) error {
		attemptCount++
		if attemptCount <= attemptsNeeded {
			return fmt.Errorf("transient error %d", attemptCount)
		}
		// Persist on success
		repo := repository.NewMessagesRepository(pool)
		return PersistHandler(repo)(ctx, evt)
	}
	w := NewWithOptions(mq, h, Options{MaxAttempts: 5, MaxAge: time.Hour, BackoffBase: 10 * time.Millisecond, BackoffCap: 20 * time.Millisecond}, nil)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go w.Start(ctx)

	evt := queue.OutboundMessageEvent{SchemaVersion: 1, Channel: queue.ChannelSMS, CustomerID: "1", From: "+155501", To: "+155502", Body: "retry success"}
	if _, err := mq.Publish(ctx, evt); err != nil {
		t.Fatalf("publish failed: %v", err)
	}

	waitForCount(t, pool, `SELECT COUNT(*) FROM messages`, 1, 3*time.Second)
	if attemptCount != attemptsNeeded+1 { // +1 for the success attempt
		t.Fatalf("expected %d total attempts, got %d", attemptsNeeded+1, attemptCount)
	}
	if w.DLQLength() != 0 {
		t.Fatalf("expected DLQ length 0, got %d", w.DLQLength())
	}
}

// TestWorker_DLQPermanentFailure ensures a permanently failing handler results in DLQ after MaxAttempts.
func TestWorker_DLQPermanentFailure(t *testing.T) {
	pool := newTestPool(t)
	resetDB(t, pool)

	mq := qmemory.New(32)
	h := func(ctx context.Context, evt queue.OutboundMessageEvent) error {
		return fmt.Errorf("permanent failure")
	}
	w := NewWithOptions(mq, h, Options{MaxAttempts: 3, MaxAge: time.Hour, BackoffBase: 5 * time.Millisecond, BackoffCap: 10 * time.Millisecond}, nil)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go w.Start(ctx)

	evt := queue.OutboundMessageEvent{SchemaVersion: 1, Channel: queue.ChannelSMS, CustomerID: "1", From: "+155511", To: "+155512", Body: "dlq failure"}
	if _, err := mq.Publish(ctx, evt); err != nil {
		t.Fatalf("publish failed: %v", err)
	}

	// Wait until DLQ length == 1
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if w.DLQLength() == 1 {
			break
		}
		time.Sleep(20 * time.Millisecond)
	}
	if w.DLQLength() != 1 {
		t.Fatalf("expected DLQ length 1, got %d", w.DLQLength())
	}
	// Ensure no rows persisted
	waitCtx, cancel2 := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel2()
	var count int
	row := pool.QueryRow(waitCtx, `SELECT COUNT(*) FROM messages`)
	_ = row.Scan(&count) // ignore scan error if table empty early
	if count != 0 {
		t.Fatalf("expected 0 persisted messages, got %d", count)
	}
}
