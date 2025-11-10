package middleware

import (
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"
)

// AuthConfig defines runtime parameters for token auth & throttling of failures.
type AuthConfig struct {
	Enabled     bool
	Tokens      []string // accepted bearer tokens
	SessionTTL  time.Duration
	MaxFailures int
	Backoff     time.Duration
}

// authState stores ephemeral session & failure counters.
type authState struct {
	mu       sync.Mutex
	sessions map[string]time.Time // token -> expiry
	failures map[string]int       // remoteAddr -> count
}

func newAuthState() *authState {
	return &authState{sessions: make(map[string]time.Time), failures: make(map[string]int)}
}

// AuthMiddleware enforces simple Bearer token auth if enabled.
// Missing or invalid token => 401; exceeded failures => 429 with Retry-After.
// Expired session => 401 requiring re-auth.
func AuthMiddleware(cfg AuthConfig) func(http.Handler) http.Handler {
	// Fast path: if disabled, return passthrough
	if !cfg.Enabled {
		return func(next http.Handler) http.Handler { return next }
	}
	accepted := make(map[string]struct{}, len(cfg.Tokens))
	for _, t := range cfg.Tokens {
		tt := strings.TrimSpace(t)
		if tt != "" {
			accepted[tt] = struct{}{}
		}
	}
	state := newAuthState()
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			token := extractBearer(r.Header.Get("Authorization"))
			if token == "" {
				writeAuthError(w, http.StatusUnauthorized, "missing bearer token")
				trackFailure(state, r.RemoteAddr, cfg)
				return
			}
			// Failure throttling check first
			if exceededFailures(state, r.RemoteAddr, cfg) {
				w.Header().Set("Retry-After", strconv.FormatInt(int64(cfg.Backoff.Seconds()), 10))
				writeAuthError(w, http.StatusTooManyRequests, "too many auth failures")
				return
			}
			// Validate token
			if _, ok := accepted[token]; !ok {
				writeAuthError(w, http.StatusUnauthorized, "invalid token")
				trackFailure(state, r.RemoteAddr, cfg)
				return
			}
			// Session expiry check
			if expired := isExpired(state, token); expired {
				writeAuthError(w, http.StatusUnauthorized, "session expired")
				// refresh session after signaling expiry to allow re-use on next attempt
				refreshSession(state, token, cfg)
				return
			}
			refreshSession(state, token, cfg)
			next.ServeHTTP(w, r)
		})
	}
}

func extractBearer(h string) string {
	h = strings.TrimSpace(h)
	if h == "" {
		return ""
	}
	if !strings.HasPrefix(strings.ToLower(h), "bearer ") {
		return ""
	}
	return strings.TrimSpace(h[7:])
}

func writeAuthError(w http.ResponseWriter, status int, msg string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_, _ = w.Write([]byte(`{"error":"` + msg + `"}`))
}

func trackFailure(st *authState, addr string, cfg AuthConfig) {
	st.mu.Lock()
	defer st.mu.Unlock()
	st.failures[addr]++
}

func exceededFailures(st *authState, addr string, cfg AuthConfig) bool {
	st.mu.Lock()
	defer st.mu.Unlock()
	return cfg.MaxFailures > 0 && st.failures[addr] >= cfg.MaxFailures
}

func refreshSession(st *authState, token string, cfg AuthConfig) {
	if cfg.SessionTTL <= 0 {
		return
	}
	st.mu.Lock()
	defer st.mu.Unlock()
	st.sessions[token] = time.Now().Add(cfg.SessionTTL)
}

func isExpired(st *authState, token string) bool {
	st.mu.Lock()
	defer st.mu.Unlock()
	if exp, ok := st.sessions[token]; ok {
		return time.Now().After(exp)
	}
	// no session yet -> treat as not expired
	return false
}

// (Removed helper functions; using strconv.FormatInt directly for Retry-After seconds)
