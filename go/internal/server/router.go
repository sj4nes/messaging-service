package server

import (
	"net/http"

	"github.com/go-chi/chi/v5"
)

// New constructs the base router with core middleware.
func New() *chi.Mux {
	r := chi.NewRouter()
	// TODO: Add request ID, logging, recovery, rate limiting, security headers.
	return r
}

// HealthHandler returns a simple handler for health checks.
func HealthHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	}
}
