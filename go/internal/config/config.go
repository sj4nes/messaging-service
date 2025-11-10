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
    cfg := &Config{
        Port:        port,
        HealthPath:  getenv("HEALTH_PATH", "/healthz"),
        LogLevel:    getenv("LOG_LEVEL", "info"),
        MetricsPath: getenv("METRICS_PATH", "/metrics"),
    }
    return cfg, nil
}
