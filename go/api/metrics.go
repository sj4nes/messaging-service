package api

import (
    "net/http"

    "github.com/sj4nes/messaging-service/go/internal/metrics"
)

// MetricsHandler adapts the internal registry to an API-level handler.
func MetricsHandler(reg *metrics.Registry) http.Handler {
    return reg.Handler()
}
