package logging

import (
	"os"
	"regexp"
	"strings"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

// Init creates a zap.Logger honoring the provided level (info, debug, warn, error)
// and applies optional redaction controlled via env:
//   - LOG_REDACT=true to enable redaction
//   - LOG_REDACT_PATTERNS=regex1,regex2 to mask matches with "***"
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
	baseCore, err := cfg.Build()
	if err != nil {
		return nil, err
	}

	if strings.EqualFold(strings.TrimSpace(os.Getenv("LOG_REDACT")), "true") {
		patterns := compilePatterns(os.Getenv("LOG_REDACT_PATTERNS"))
		core := newRedactingCore(baseCore.Core(), patterns)
		// Recreate logger with redacting core; attach same fields (no direct Options accessor available)
		return zap.New(core, zap.AddCaller(), zap.AddStacktrace(cfg.Level)), nil
	}
	return baseCore, nil
}

// compilePatterns compiles comma-separated regex list, with a few safe defaults if empty.
func compilePatterns(csv string) []*regexp.Regexp {
	var ps []string
	csv = strings.TrimSpace(csv)
	if csv == "" {
		ps = []string{`(?i)authorization: .*`, `(?i)api[_-]?key=\w+`, `(?i)token=\w+`}
	} else {
		ps = strings.Split(csv, ",")
	}
	res := make([]*regexp.Regexp, 0, len(ps))
	for _, p := range ps {
		p = strings.TrimSpace(p)
		if p == "" {
			continue
		}
		if r, err := regexp.Compile(p); err == nil {
			res = append(res, r)
		}
	}
	return res
}

// redactingCore wraps a Core and redacts entry message and string fields.
type redactingCore struct {
	zapcore.Core
	patterns []*regexp.Regexp
}

func newRedactingCore(inner zapcore.Core, patterns []*regexp.Regexp) *redactingCore {
	return &redactingCore{Core: inner, patterns: patterns}
}

func (c *redactingCore) With(fields []zapcore.Field) zapcore.Core {
	return &redactingCore{Core: c.Core.With(c.redactFields(fields)), patterns: c.patterns}
}

func (c *redactingCore) Check(ent zapcore.Entry, ce *zapcore.CheckedEntry) *zapcore.CheckedEntry {
	ent.Message = c.redactString(ent.Message)
	return c.Core.Check(ent, ce)
}

func (c *redactingCore) Write(ent zapcore.Entry, fields []zapcore.Field) error {
	ent.Message = c.redactString(ent.Message)
	return c.Core.Write(ent, c.redactFields(fields))
}

func (c *redactingCore) redactString(s string) string {
	if s == "" || len(c.patterns) == 0 {
		return s
	}
	out := s
	for _, r := range c.patterns {
		out = r.ReplaceAllString(out, "***")
	}
	return out
}

func (c *redactingCore) redactFields(fields []zapcore.Field) []zapcore.Field {
	if len(fields) == 0 || len(c.patterns) == 0 {
		return fields
	}
	redacted := make([]zapcore.Field, 0, len(fields))
	for _, f := range fields {
		switch f.Type {
		case zapcore.StringType:
			redacted = append(redacted, zapcore.Field{Key: f.Key, Type: zapcore.StringType, String: c.redactString(f.String)})
		default:
			redacted = append(redacted, f)
		}
	}
	return redacted
}
