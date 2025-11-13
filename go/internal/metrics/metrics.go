package metrics

import (
	"net/http"
	"sync/atomic"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

// Registry exposes a prometheus registry for custom metric registration.
type Registry struct {
	reg             *prometheus.Registry
	workerProcessed prometheus.Counter
	started         int64 // gauge-like via atomic load for quick introspection if needed
}

func NewRegistry() *Registry {
	reg := prometheus.NewRegistry()
	wp := prometheus.NewCounter(prometheus.CounterOpts{Name: "worker_processed", Help: "Count of messages processed (accepted)"})
	_ = reg.Register(wp)
	r := &Registry{reg: reg, workerProcessed: wp}
	atomic.StoreInt64(&r.started, 1)
	return r
}

// Handler returns an http.Handler that serves metrics.
func (r *Registry) Handler() http.Handler {
	return promhttp.HandlerFor(r.reg, promhttp.HandlerOpts{})
}

// Register allows adding collectors.
func (r *Registry) Register(c prometheus.Collector) error {
	return r.reg.Register(c)
}

// IncWorkerProcessed increments the worker_processed counter.
func (r *Registry) IncWorkerProcessed() { r.workerProcessed.Inc() }
