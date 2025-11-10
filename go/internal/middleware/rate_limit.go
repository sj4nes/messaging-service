package middleware

import (
	"net/http"
	"sync"

	"golang.org/x/time/rate"
)

// RateLimiterConfig basic configuration.
type RateLimiterConfig struct {
	RequestsPerSecond float64
	Burst             int
}

// keyLimiter stores per-key limiters.
type keyLimiter struct {
	mu       sync.Mutex
	limiters map[string]*rate.Limiter
	cfg      RateLimiterConfig
}

func newKeyLimiter(cfg RateLimiterConfig) *keyLimiter {
	return &keyLimiter{limiters: make(map[string]*rate.Limiter), cfg: cfg}
}

func (k *keyLimiter) get(key string) *rate.Limiter {
	k.mu.Lock()
	defer k.mu.Unlock()
	l, ok := k.limiters[key]
	if !ok {
		l = rate.NewLimiter(rate.Limit(k.cfg.RequestsPerSecond), k.cfg.Burst)
		k.limiters[key] = l
	}
	return l
}

// RateLimit middleware using remote address as key (placeholder).
func RateLimit(cfg RateLimiterConfig) func(http.Handler) http.Handler {
	store := newKeyLimiter(cfg)
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			key := r.RemoteAddr // TODO: replace with extracted client identifier
			limiter := store.get(key)
			if !limiter.Allow() {
				w.Header().Set("Retry-After", "1")
				w.WriteHeader(http.StatusTooManyRequests)
				_, _ = w.Write([]byte("rate limit exceeded"))
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}

// Background cleanup could be added later to reap old limiters.
// For now, rely on process lifetime; entries are small.
