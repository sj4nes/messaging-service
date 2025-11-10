package server

import (
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/sj4nes/messaging-service/go/internal/middleware"
)

// New constructs the base router with core middleware.
func New() *chi.Mux {
	r := chi.NewRouter()
	// Baseline middleware (enhance later: request ID, logging, recovery)
	return r
}

// WithProtected applies auth + security headers + rate limiting to protected routes and registers handlers via fn.
func WithProtected(r *chi.Mux, auth middleware.AuthConfig, protectedRL middleware.RateLimiterConfig, fn func(gr chi.Router)) {
	r.Group(func(gr chi.Router) {
		gr.Use(middleware.SecurityHeaders)
		gr.Use(middleware.RateLimit(protectedRL))
		gr.Use(middleware.AuthMiddleware(auth))
		fn(gr)
	})
}

// HealthHandler returns a simple handler for health checks.
func HealthHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	}
}
