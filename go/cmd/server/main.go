package main

import (
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"strings"

	"github.com/prometheus/client_golang/prometheus"
	"go.uber.org/zap"

	"github.com/sj4nes/messaging-service/go/internal/config"
	"github.com/sj4nes/messaging-service/go/internal/logging"
	"github.com/sj4nes/messaging-service/go/internal/metrics"
	"github.com/sj4nes/messaging-service/go/internal/server"
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
	// Example custom metric placeholder
	requests := prometheus.NewCounter(prometheus.CounterOpts{Name: "app_startups_total", Help: "Number of application startups"})
	_ = reg.Register(requests)
	requests.Inc()

	r := server.New()
	r.Get(cfg.HealthPath, server.HealthHandler())
	r.Handle(cfg.MetricsPath, reg.Handler())

	addr := fmt.Sprintf(":%d", cfg.Port)
	log.Info("starting server", zap.Int("port", cfg.Port), zap.String("health", cfg.HealthPath), zap.String("metrics", cfg.MetricsPath))
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
func defaultAddr() string {
    port := os.Getenv("PORT")
    if strings.TrimSpace(port) == "" {
        return ":8080"
    }
    return ":" + port
}

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
