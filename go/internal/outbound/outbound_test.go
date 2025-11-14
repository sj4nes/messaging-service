package outbound

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"

	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	"github.com/sj4nes/messaging-service/go/internal/metrics"
	"github.com/sj4nes/messaging-service/go/internal/config"
	"github.com/sj4nes/messaging-service/go/internal/providers"
	mockprov "github.com/sj4nes/messaging-service/go/internal/providers/mock"
	"github.com/sj4nes/messaging-service/go/internal/queue"
	qmemory "github.com/sj4nes/messaging-service/go/internal/queue/memory"
	"github.com/sj4nes/messaging-service/go/internal/state"
	"github.com/sj4nes/messaging-service/go/internal/resilience"
	"github.com/sj4nes/messaging-service/go/internal/worker"
)

func newTestPool(t *testing.T) *pgxpool.Pool {
	t.Helper()
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		t.Skip("DATABASE_URL not set; skipping outbound integration tests")
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

func TestOutbound_RoutesAndUpdatesProvider(t *testing.T) {
	pool := newTestPool(t)

	// truncate tables and seed provider
	ctx := context.Background()
	if _, err := pool.Exec(ctx, `TRUNCATE TABLE messages RESTART IDENTITY CASCADE`); err != nil {
		t.Fatalf("truncate failed: %v", err)
	}
	if _, err := pool.Exec(ctx, `TRUNCATE TABLE conversations RESTART IDENTITY CASCADE`); err != nil {
		t.Fatalf("truncate convo failed: %v", err)
	}
	if _, err := pool.Exec(ctx, `TRUNCATE TABLE providers RESTART IDENTITY CASCADE`); err != nil {
		t.Fatalf("truncate providers failed: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO customers (id, name) VALUES (1, 'Test') ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name`); err != nil {
		t.Fatalf("seed customer failed: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO providers (id, customer_id, kind, name) VALUES (1, 1, 'sms', 'sms-mms')`); err != nil {
		t.Fatalf("seed provider failed: %v", err)
	}

	mq := qmemory.New(32)
	repo := repository.NewMessagesRepository(pool)
	// Insert an outbound message in DB to get its id
	msgID, err := repo.InsertOutbound(ctx, "sms", "+15550001", "+15550002", "Hello", time.Now().UTC().Format(time.RFC3339))
	if err != nil {
		t.Fatalf("InsertOutbound failed: %v", err)
	}

	// Provider registry
	pr := providers.NewRegistry()
	// pass a default config so the mock provider can pick outcomes based on env-driven seed
	pr.Insert(providers.ChannelSms, mockprov.NewSmsMmsProvider(&config.Config{}))

	pb := state.NewProviderBreakers()
	pb.Insert("sms-mms", resilience.New("sms-mms"))

	reg := metrics.NewRegistry()

	handler := DispatchHandler(pr, pb, repo, reg)
	w := worker.New(mq, handler)
	ctxC, cancel := context.WithCancel(context.Background())
	defer cancel()
	go w.Start(ctxC)

	// Publish an outbound event with message_id set so dispatcher can tag DB.
	evt := queue.OutboundMessageEvent{SchemaVersion: 1, Channel: queue.ChannelSMS, CustomerID: "1", From: "+15550001", To: "+15550002", Body: "Hello", Metadata: map[string]any{"message_id": msgID}}
	if _, err := mq.Publish(ctx, evt); err != nil {
		t.Fatalf("Publish failed: %v", err)
	}

	// Wait for DB update by polling
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		var pid int
		r := pool.QueryRow(ctx, `SELECT provider_id FROM messages WHERE id::text = $1`, msgID)
		if err := r.Scan(&pid); err == nil {
			if pid != 0 { // provider_id set
				return
			}
		}
		time.Sleep(20 * time.Millisecond)
	}
	t.Fatalf("provider not set on message %s", msgID)
}
