package config

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

// Config holds application configuration loaded from environment.
type Config struct {
	Port        int
	HealthPath  string
	LogLevel    string
	MetricsPath string

	// Security / Auth
	AuthEnabled           bool
	AuthTokens            []string // static shared secrets (Bearer tokens) for parity baseline
	AuthSessionTTLSeconds int
	AuthMaxFailures       int
	AuthBackoffSeconds    int

	// Rate limits (public vs protected)
	PublicRPS      float64
	PublicBurst    int
	ProtectedRPS   float64
	ProtectedBurst int

	// SSRF allowlist hosts
	SSRFAllowlist []string
	// Debug / profiling
	PprofEnabled bool
	PprofPath    string

	// Worker / Queue reliability options
	WorkerMaxAttempts   int
	WorkerMaxAgeHours   int
	WorkerBackoffBaseMs int
	WorkerBackoffCapMs  int

	// Provider mock config
	ProviderTimeoutPct   int
	ProviderErrorPct     int
	ProviderRatelimitPct int
	ProviderSeed         int64
	ProviderSmsSeed      int64
	ProviderEmailSeed    int64
}

func getenv(key, def string) string {
	if v := strings.TrimSpace(os.Getenv(key)); v != "" {
		return v
	}
	return def
}

// Load reads environment variables into a Config instance.
func Load() (*Config, error) {
	portStr := getenv("PORT", "8080")
	port, err := strconv.Atoi(portStr)
	if err != nil || port <= 0 {
		return nil, fmt.Errorf("invalid PORT: %q", portStr)
	}
	// Auth & security
	authEnabled := strings.EqualFold(getenv("AUTH_ENABLED", "false"), "true")
	tokensCSV := getenv("AUTH_TOKENS", "")
	var tokens []string
	if tokensCSV != "" {
		for _, t := range strings.Split(tokensCSV, ",") {
			tt := strings.TrimSpace(t)
			if tt != "" {
				tokens = append(tokens, tt)
			}
		}
	}
	ttlSeconds := atoiDefault(getenv("AUTH_SESSION_TTL_SECONDS", "3600"), 3600)
	maxFailures := atoiDefault(getenv("AUTH_MAX_FAILURES", "5"), 5)
	backoffSeconds := atoiDefault(getenv("AUTH_BACKOFF_SECONDS", "2"), 2)

	// Rate limits
	publicRPS := atofDefault(getenv("RATE_LIMIT_PUBLIC_RPS", "5"), 5)
	publicBurst := atoiDefault(getenv("RATE_LIMIT_PUBLIC_BURST", "10"), 10)
	protectedRPS := atofDefault(getenv("RATE_LIMIT_PROTECTED_RPS", "2"), 2)
	protectedBurst := atoiDefault(getenv("RATE_LIMIT_PROTECTED_BURST", "5"), 5)

	// SSRF allowlist
	allowCSV := getenv("SSRF_ALLOWLIST", "example.com")
	var allowlist []string
	if allowCSV != "" {
		for _, h := range strings.Split(allowCSV, ",") {
			hh := strings.TrimSpace(h)
			if hh != "" {
				allowlist = append(allowlist, hh)
			}
		}
	}

	cfg := &Config{
		Port:                  port,
		HealthPath:            getenv("HEALTH_PATH", "/healthz"),
		LogLevel:              getenv("LOG_LEVEL", "info"),
		MetricsPath:           getenv("METRICS_PATH", "/metrics"),
		AuthEnabled:           authEnabled,
		AuthTokens:            tokens,
		AuthSessionTTLSeconds: ttlSeconds,
		AuthMaxFailures:       maxFailures,
		AuthBackoffSeconds:    backoffSeconds,
		PublicRPS:             publicRPS,
		PublicBurst:           publicBurst,
		ProtectedRPS:          protectedRPS,
		ProtectedBurst:        protectedBurst,
		SSRFAllowlist:         allowlist,
		PprofEnabled:          strings.EqualFold(getenv("PPROF_ENABLED", "false"), "true"),
		PprofPath:             getenv("PPROF_PATH", "/debug/pprof"),
		WorkerMaxAttempts:     atoiDefault(getenv("WORKER_MAX_ATTEMPTS", "10"), 10),
		WorkerMaxAgeHours:     atoiDefault(getenv("WORKER_MAX_AGE_HOURS", "72"), 72),
		WorkerBackoffBaseMs:   atoiDefault(getenv("WORKER_BACKOFF_BASE_MS", "200"), 200),
		WorkerBackoffCapMs:    atoiDefault(getenv("WORKER_BACKOFF_CAP_MS", "5000"), 5000),
		ProviderTimeoutPct:    atoiDefault(getenv("PROVIDER_TIMEOUT_PCT", "0"), 0),
		ProviderErrorPct:      atoiDefault(getenv("PROVIDER_ERROR_PCT", "0"), 0),
		ProviderRatelimitPct:  atoiDefault(getenv("PROVIDER_RATELIMIT_PCT", "0"), 0),
		ProviderSeed:          atollDefault(getenv("PROVIDER_SEED", "0"), 0),
		ProviderSmsSeed:       atollDefault(getenv("PROVIDER_SMS_SEED", "0"), 0),
		ProviderEmailSeed:     atollDefault(getenv("PROVIDER_EMAIL_SEED", "0"), 0),
	}
	return cfg, nil
}

func atoiDefault(raw string, def int) int {
	if v, err := strconv.Atoi(strings.TrimSpace(raw)); err == nil && v > 0 {
		return v
	}
	return def
}

func atofDefault(raw string, def float64) float64 {
	if v, err := strconv.ParseFloat(strings.TrimSpace(raw), 64); err == nil && v > 0 {
		return v
	}
	return def
}

func atollDefault(raw string, def int64) int64 {
	if v, err := strconv.ParseInt(strings.TrimSpace(raw), 10, 64); err == nil && v > 0 {
		return v
	}
	return def
}
