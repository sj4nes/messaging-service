package logging

import (
	"strings"

	"go.uber.org/zap"
)

// Init creates a zap.Logger honoring the provided level (info, debug, warn, error).
func Init(level string) (*zap.Logger, error) {
	lvl := strings.ToLower(strings.TrimSpace(level))
	cfg := zap.NewProductionConfig()
	switch lvl {
	case "debug":
		cfg.Level = zap.NewAtomicLevelAt(zap.DebugLevel)
	case "warn":
		cfg.Level = zap.NewAtomicLevelAt(zap.WarnLevel)
	case "error":
		cfg.Level = zap.NewAtomicLevelAt(zap.ErrorLevel)
	default:
		cfg.Level = zap.NewAtomicLevelAt(zap.InfoLevel)
	}
	return cfg.Build()
}
