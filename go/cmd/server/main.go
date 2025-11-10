package main

import (
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"strings"

	"github.com/go-chi/chi/v5"
)

func main() {
	r := chi.NewRouter()

	// Basic health endpoint
	r.Get("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})

	addr := defaultAddr()
	fmt.Printf("starting server on %s (set PORT env to override)\n", addr)
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
