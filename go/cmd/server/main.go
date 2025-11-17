package main

import (
	"context"
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/prometheus/client_golang/prometheus"
	"go.uber.org/zap"

	"github.com/sj4nes/messaging-service/go/api"
	"github.com/sj4nes/messaging-service/go/internal/config"
	"github.com/sj4nes/messaging-service/go/internal/db/migrate"
	"github.com/sj4nes/messaging-service/go/internal/db/repository"
	dbstore "github.com/sj4nes/messaging-service/go/internal/db/store"
	"github.com/sj4nes/messaging-service/go/internal/logging"
	"github.com/sj4nes/messaging-service/go/internal/metrics"
	"github.com/sj4nes/messaging-service/go/internal/middleware"
	"github.com/sj4nes/messaging-service/go/internal/outbound"
	"github.com/sj4nes/messaging-service/go/internal/providers"
	mock "github.com/sj4nes/messaging-service/go/internal/providers/mock"
	qmemory "github.com/sj4nes/messaging-service/go/internal/queue/memory"
	"github.com/sj4nes/messaging-service/go/internal/resilience"
	"github.com/sj4nes/messaging-service/go/internal/server"
	"github.com/sj4nes/messaging-service/go/internal/state"
	"github.com/sj4nes/messaging-service/go/internal/worker"
)

func main() {
	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "configuration error: %v\n", err)
		os.Exit(2)
	}

	log, err := logging.Init(cfg.LogLevel)
	if err != nil {
		fmt.Fprintf(os.Stderr, "logger init failed: %v\n", err)
		os.Exit(2)
	}
	defer log.Sync() // flush

	reg := metrics.NewRegistry()
	// Retain startup metric for observability (optional)
	startups := prometheus.NewCounter(prometheus.CounterOpts{Name: "app_startups_total", Help: "Number of application startups"})
	_ = reg.Register(startups)
	startups.Inc()

	// Optional migrations (dev/CI): RUN_DB_MIGRATIONS=true
	if strings.EqualFold(os.Getenv("RUN_DB_MIGRATIONS"), "true") {
		mdir := os.Getenv("MIGRATIONS_DIR")
		if strings.TrimSpace(mdir) == "" {
			mdir = "../crates/db-migrate/migrations_sqlx"
		}
		dbURL := os.Getenv("DATABASE_URL")
		if strings.TrimSpace(dbURL) == "" {
			log.Warn("RUN_DB_MIGRATIONS set but DATABASE_URL missing; skipping migrations")
		} else {
			log.Info("applying database migrations", zap.String("path", mdir))
			if err := migrate.ApplyUp(context.Background(), mdir, dbURL); err != nil {
				log.Warn("migration apply failed (continuing)", zap.Error(err))
			}
		}
	}

	r := server.New()
	// Public middleware (applies to all requests before protected grouping)
	r.Use(middleware.SecurityHeaders)
	r.Use(middleware.RateLimit(middleware.RateLimiterConfig{RequestsPerSecond: cfg.PublicRPS, Burst: cfg.PublicBurst}))

	// Public endpoints: health & metrics remain unauthenticated for operability
	r.Get(cfg.HealthPath, api.HealthHandler())
	r.Handle(cfg.MetricsPath, api.MetricsHandler(reg))

	// Optional: if DATABASE_URL is provided, initialize DB store (conversations backed by Postgres)
	// If API_ENABLE_INMEMORY_FALLBACK=true, prefer in-memory store even when DATABASE_URL is supplied.
	dbURL := strings.TrimSpace(os.Getenv("DATABASE_URL"))
	log.Info("database configuration", zap.Bool("has_database_url", dbURL != ""), zap.Bool("inmemory_fallback", cfg.EnableInmemoryFallback))
	if dbURL != "" && !cfg.EnableInmemoryFallback {
		pool, err := pgxpool.New(context.Background(), dbURL)
		if err != nil {
			log.Warn("db pool init failed; using in-memory store", zap.Error(err))
		} else {
			// Initialize in-memory queue for enqueueing (US1). Worker wiring is added later.
			mq := qmemory.New(1024)
			api.SetStore(dbstore.NewWithQueue(pool, mq))
			// Start worker with persistence handler and configured options.
			msgRepo := repository.NewMessagesRepository(pool)
			wOpts := worker.Options{
				MaxAttempts: cfg.WorkerMaxAttempts,
				MaxAge:      time.Duration(cfg.WorkerMaxAgeHours) * time.Hour,
				BackoffBase: time.Duration(cfg.WorkerBackoffBaseMs) * time.Millisecond,
				BackoffCap:  time.Duration(cfg.WorkerBackoffCapMs) * time.Millisecond,
			}
			w := worker.NewWithOptions(mq, worker.PersistHandler(msgRepo), wOpts, reg)
			go w.Start(context.Background())
			// Provider registry & per-provider breakers (Feature 008 parity with Rust)
			provReg := providers.NewRegistry()
			provReg.Insert(providers.ChannelSms, mock.NewSmsMmsProvider(cfg))
			provReg.Insert(providers.ChannelMms, mock.NewSmsMmsProvider(cfg))
			provReg.Insert(providers.ChannelEmail, mock.NewEmailProvider(cfg))

			pb := state.NewProviderBreakers()
			pb.Insert("sms-mms", resilience.New("sms-mms"))
			pb.Insert("email", resilience.New("email"))

			// Start an outbound worker that routes to providers and simulates dispatch
			outboundHandler := outbound.DispatchHandler(provReg, pb, msgRepo, reg)
			w2 := worker.New(mq, outboundHandler)
			go w2.Start(context.Background())
			
			log.Info("db-backed store enabled", zap.Bool("messages", true), zap.Bool("conversations", true))
			defer pool.Close()
		}
	} else if cfg.EnableInmemoryFallback {
		// Prefer Go-specific env var name in the message if it was set, otherwise show the legacy var
		var envUsed string
		if strings.TrimSpace(os.Getenv("GO_API_ENABLE_INMEMORY_FALLBACK")) != "" {
			envUsed = "GO_API_ENABLE_INMEMORY_FALLBACK"
		} else {
			envUsed = "API_ENABLE_INMEMORY_FALLBACK"
		}
		log.Info("in-memory store enabled for self-test (env)", zap.String("env", envUsed))
	}

	// Protected endpoints grouping (messages, conversations, webhooks)
	server.WithProtected(r, middleware.AuthConfig{
		Enabled:     cfg.AuthEnabled,
		Tokens:      cfg.AuthTokens,
		SessionTTL:  time.Duration(cfg.AuthSessionTTLSeconds) * time.Second,
		MaxFailures: cfg.AuthMaxFailures,
		Backoff:     time.Duration(cfg.AuthBackoffSeconds) * time.Second,
	}, middleware.RateLimiterConfig{RequestsPerSecond: cfg.ProtectedRPS, Burst: cfg.ProtectedBurst}, func(gr chi.Router) {
		// Messaging API routes (User Story 1 subset) - protected when auth enabled
		api.Routes(gr)
	})

	// Optional pprof endpoints (operability)
	if cfg.PprofEnabled {
		server.MountPprof(r, cfg.PprofPath)
		log.Info("pprof enabled", zap.String("path", cfg.PprofPath))
	}

	addr := fmt.Sprintf(":%d", cfg.Port)
	log.Info("starting server", zap.Int("port", cfg.Port), zap.String("health", cfg.HealthPath), zap.String("metrics", cfg.MetricsPath), zap.Bool("auth_enabled", cfg.AuthEnabled))
	if err := http.ListenAndServe(addr, r); err != nil {
		if isAddrInUse(err) {
			fmt.Fprintf(os.Stderr, "ERROR: address %s already in use. Suggestions:\n", addr)
			fmt.Fprintf(os.Stderr, "  * Export PORT to a free port: PORT=8081 make go.run\n")
			fmt.Fprintf(os.Stderr, "  * Or terminate the existing process: lsof -iTCP%s -sTCP:LISTEN\n", addr)
			os.Exit(2)
		}
		fmt.Fprintf(os.Stderr, "server error: %v\n", err)
		os.Exit(1)
	}
}

// defaultAddr returns the listen address, using PORT env var if set.
// (Deprecated) defaultAddr retained for compatibility but unused after config loader integration.
// NOTE: legacy defaultAddr removed; config loader provides port.

// isAddrInUse detects common 'address already in use' errors across platforms.
func isAddrInUse(err error) bool {
	if err == nil {
		return false
	}
	var opErr *net.OpError
	if errors.As(err, &opErr) {
		if opErr.Err != nil && strings.Contains(strings.ToLower(opErr.Err.Error()), "address already in use") {
			return true
		}
	}
	// Fallback string match
	return strings.Contains(strings.ToLower(err.Error()), "address already in use")
}
