package seed

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5/pgxpool"
)

// SeedMinimumIfNeeded inserts a minimal conversation + messages if the DB is empty.
// It is idempotent and safe to call on every list operation.
// Always checks DB state rather than relying on in-memory flags so it works after DB resets.
func SeedMinimumIfNeeded(ctx context.Context, pool *pgxpool.Pool) {
	if pool == nil {
		return
	}
	
	// Check if migrations have been applied by verifying customers table exists
	var tableExists bool
	err := pool.QueryRow(ctx, `SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'customers')`).Scan(&tableExists)
	if err != nil || !tableExists {
		// Migrations not yet applied, skip seed (this is expected during initial startup)
		return
	}
	
	// First, ensure required foreign key dependencies exist: customers and providers
	// InsertOutbound hardcodes customer_id=1 and provider_id=1, so we need those rows
	fmt.Println("SeedMinimumIfNeeded: checking/creating baseline customers and providers...")
	
	// Insert baseline customer (id=1) if missing
	_, err = pool.Exec(ctx, `INSERT INTO customers (id, name, created_at, updated_at) 
		VALUES (1, 'test-customer', now(), now()) 
		ON CONFLICT (id) DO NOTHING`)
	if err != nil {
		fmt.Printf("SeedMinimumIfNeeded: failed to insert customer: %v\n", err)
		return
	}
	
	// Insert baseline provider (id=1) if missing - providers table requires customer_id, kind, and name
	_, err = pool.Exec(ctx, `INSERT INTO providers (id, customer_id, kind, name, created_at, updated_at) 
		VALUES (1, 1, 'sms', 'test-provider', now(), now()) 
		ON CONFLICT (id) DO NOTHING`)
	if err != nil {
		fmt.Printf("SeedMinimumIfNeeded: failed to insert provider: %v\n", err)
		return
	}
	
	fmt.Println("SeedMinimumIfNeeded: baseline dependencies ready")
	
	// Check if conversation id=1 already exists
	var convExists bool
	if err := pool.QueryRow(ctx, `SELECT EXISTS(SELECT 1 FROM conversations WHERE id = 1)`).Scan(&convExists); err != nil {
		return // silent; not critical
	}
	if convExists {
		return
	}
	
	// Create conversation id=1 explicitly for legacy test compatibility
	fmt.Println("SeedMinimumIfNeeded: creating baseline conversation id=1")
	_, err = pool.Exec(ctx, `
		INSERT INTO conversations (id, customer_id, topic, channel, participant_a, participant_b, message_count, last_activity_at, key)
		VALUES (1, 1, NULL, 'sms', '+15550001', '+15550002', 0, now(), NULL)
		ON CONFLICT (id) DO NOTHING
	`)
	if err != nil {
		fmt.Printf("SeedMinimumIfNeeded: failed to create conversation 1: %v\n", err)
		return
	}
	
	// Insert baseline body for deduplication
	var bodyID int64
	err = pool.QueryRow(ctx, `
		INSERT INTO message_bodies (body) VALUES ('Hello there') 
		ON CONFLICT (body) DO UPDATE SET body = EXCLUDED.body 
		RETURNING id
	`).Scan(&bodyID)
	if err != nil {
		fmt.Printf("SeedMinimumIfNeeded: failed to insert message body: %v\n", err)
		return
	}
	
	// Insert messages for conversation 1
	_, err = pool.Exec(ctx, `
		INSERT INTO messages (conversation_id, provider_id, direction, sent_at, received_at, body_id)
		VALUES 
			(1, 1, 'outbound', now(), now(), $1),
			(1, 1, 'inbound', now(), now(), $1)
		ON CONFLICT DO NOTHING
	`, bodyID)
	if err != nil {
		fmt.Printf("SeedMinimumIfNeeded: failed to insert messages: %v\n", err)
		return
	}
	
	// Update message count
	_, _ = pool.Exec(ctx, `UPDATE conversations SET message_count = (SELECT COUNT(*) FROM messages WHERE conversation_id = 1), last_activity_at = now() WHERE id = 1`)
	
	// Update sequences to avoid conflicts with seeded IDs
	_, _ = pool.Exec(ctx, `SELECT setval('conversations_id_seq', (SELECT MAX(id) FROM conversations))`)
	_, _ = pool.Exec(ctx, `SELECT setval('messages_id_seq', (SELECT MAX(id) FROM messages))`)
	_, _ = pool.Exec(ctx, `SELECT setval('message_bodies_id_seq', (SELECT MAX(id) FROM message_bodies))`)
	
	fmt.Println("SeedMinimumIfNeeded: baseline conversation 1 created with messages")
}
