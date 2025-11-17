package api_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/jackc/pgx/v5/pgxpool"

	"github.com/sj4nes/messaging-service/go/api"
	"github.com/sj4nes/messaging-service/go/api/models"
	dbstore "github.com/sj4nes/messaging-service/go/internal/db/store"
	qmemory "github.com/sj4nes/messaging-service/go/internal/queue/memory"
)

// newTestRouterWithDB wires up a chi router with Routes and a DB-backed store.
// It applies migrations once per process and reuses the same database.
func newTestRouterWithDB(t *testing.T) (*chi.Mux, *pgxpool.Pool, *qmemory.MemoryQueue) {
	t.Helper()
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		t.Skip("DATABASE_URL not set; skipping HTTP + DB integration tests")
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	pool, err := pgxpool.New(ctx, url)
	if err != nil {
		t.Fatalf("failed to create pgx pool: %v", err)
	}
	t.Cleanup(func() { pool.Close() })

	mq := qmemory.New(128)
	api.SetStore(dbstore.NewWithQueue(pool, mq))

	r := chi.NewRouter()
	api.Routes(r)
	return r, pool, mq
}

// resetHTTPTestDB truncates core tables between HTTP tests.
func resetHTTPTestDB(t *testing.T, pool *pgxpool.Pool) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	stmts := []string{
		"TRUNCATE TABLE messages RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE conversations RESTART IDENTITY CASCADE",
		"TRUNCATE TABLE message_bodies RESTART IDENTITY CASCADE",
	}
	for _, stmt := range stmts {
		if _, err := pool.Exec(ctx, stmt); err != nil {
			t.Fatalf("failed to truncate with %q: %v", stmt, err)
		}
	}
}

// TestSmsHandler_AcceptsAndPersists verifies the happy path for /api/messages/sms.

func TestSmsHandler_AcceptsAndEnqueues(t *testing.T) {
	r, pool, mq := newTestRouterWithDB(t)
	resetHTTPTestDB(t, pool)

	reqBody := models.SmsRequest{
		Type: "sms",
		From: "+15551234567",
		To:   "+15557654321",
		Body: "hello via HTTP test",
	}
	data, err := json.Marshal(reqBody)
	if err != nil {
		t.Fatalf("failed to marshal request: %v", err)
	}

	req := httptest.NewRequest(http.MethodPost, "/api/messages/sms", bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	if w.Code != http.StatusAccepted {
		t.Fatalf("expected status %d, got %d, body: %s", http.StatusAccepted, w.Code, w.Body.String())
	}
	// Verify that an event was enqueued.
	if mq.Depth() != 1 {
		t.Fatalf("expected queue depth 1, got %d", mq.Depth())
	}
}

// TestSmsHandler_ValidatesType verifies that invalid type is rejected.
func TestSmsHandler_ValidatesType(t *testing.T) {
	r, _, _ := newTestRouterWithDB(t)

	reqBody := models.SmsRequest{
		Type: "invalid",
		From: "+15551234567",
		To:   "+15557654321",
		Body: "ignored",
	}
	data, err := json.Marshal(reqBody)
	if err != nil {
		t.Fatalf("failed to marshal request: %v", err)
	}

	req := httptest.NewRequest(http.MethodPost, "/api/messages/sms", bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected status %d, got %d", http.StatusBadRequest, w.Code)
	}
}

// TestEmailHandler_AcceptsAndPersists verifies /api/messages/email happy path.
func TestEmailHandler_AcceptsAndEnqueues(t *testing.T) {
	r, pool, mq := newTestRouterWithDB(t)
	resetHTTPTestDB(t, pool)

	reqBody := models.EmailRequest{
		From: "from@example.com",
		To:   "to@example.com",
		Body: "body",
	}
	data, err := json.Marshal(reqBody)
	if err != nil {
		t.Fatalf("failed to marshal request: %v", err)
	}

	req := httptest.NewRequest(http.MethodPost, "/api/messages/email", bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	if w.Code != http.StatusAccepted {
		t.Fatalf("expected status %d, got %d, body: %s", http.StatusAccepted, w.Code, w.Body.String())
	}
	if mq.Depth() != 1 {
		t.Fatalf("expected queue depth 1, got %d", mq.Depth())
	}
}

// TestEmailHandler_EmptyBody validates that input body is required.
func TestEmailHandler_EmptyBody(t *testing.T) {
	r, _, _ := newTestRouterWithDB(t)

	reqBody := models.EmailRequest{
		From: "from@example.com",
		To:   "to@example.com",
		Body: "",
	}
	data, err := json.Marshal(reqBody)
	if err != nil {
		t.Fatalf("failed to marshal request: %v", err)
	}

	req := httptest.NewRequest(http.MethodPost, "/api/messages/email", bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected status %d, got %d", http.StatusBadRequest, w.Code)
	}
}
