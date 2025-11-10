package metrics

import (
    "net/http"

    "github.com/prometheus/client_golang/prometheus"
    "github.com/prometheus/client_golang/prometheus/promhttp"
)

// Registry exposes a prometheus registry for custom metric registration.
type Registry struct {
    reg *prometheus.Registry
}

func NewRegistry() *Registry {
    return &Registry{reg: prometheus.NewRegistry()}
}

// Handler returns an http.Handler that serves metrics.
func (r *Registry) Handler() http.Handler {
    return promhttp.HandlerFor(r.reg, promhttp.HandlerOpts{})
}

// Register allows adding collectors.
func (r *Registry) Register(c prometheus.Collector) error {
    return r.reg.Register(c)
}
 
