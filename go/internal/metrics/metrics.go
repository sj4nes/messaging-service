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
	enqueueAttempt  prometheus.Counter
	enqueueSuccess  prometheus.Counter
	enqueueFailure  prometheus.Counter
	queueDepth      prometheus.Gauge
	retryTotal      prometheus.Counter
	dlqTotal        prometheus.Counter
	started         int64 // gauge-like via atomic load for quick introspection if needed
}

func NewRegistry() *Registry {
	reg := prometheus.NewRegistry()
	wp := prometheus.NewCounter(prometheus.CounterOpts{Name: "worker_processed", Help: "Count of messages processed (accepted)"})
	enqA := prometheus.NewCounter(prometheus.CounterOpts{Name: "enqueue_attempt_total", Help: "How many enqueue attempts were made"})
	enqS := prometheus.NewCounter(prometheus.CounterOpts{Name: "enqueue_success_total", Help: "How many enqueue operations succeeded"})
	enqF := prometheus.NewCounter(prometheus.CounterOpts{Name: "enqueue_failure_total", Help: "How many enqueue operations failed"})
	qd := prometheus.NewGauge(prometheus.GaugeOpts{Name: "queue_depth", Help: "Approximate depth of input-events queue"})
	_ = reg.Register(wp)
	_ = reg.Register(enqA)
	_ = reg.Register(enqS)
	_ = reg.Register(enqF)
	_ = reg.Register(qd)
	rty := prometheus.NewCounter(prometheus.CounterOpts{Name: "retry_total", Help: "Total number of worker retries"})
	dlq := prometheus.NewCounter(prometheus.CounterOpts{Name: "dlq_total", Help: "Total number of events moved to DLQ"})
	_ = reg.Register(rty)
	_ = reg.Register(dlq)
	r := &Registry{reg: reg, workerProcessed: wp, enqueueAttempt: enqA, enqueueSuccess: enqS, enqueueFailure: enqF, queueDepth: qd, retryTotal: rty, dlqTotal: dlq}
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

// Enqueue metrics
func (r *Registry) IncEnqueueAttempt()  { r.enqueueAttempt.Inc() }
func (r *Registry) IncEnqueueSuccess()  { r.enqueueSuccess.Inc() }
func (r *Registry) IncEnqueueFailure()  { r.enqueueFailure.Inc() }
func (r *Registry) SetQueueDepth(n int) { r.queueDepth.Set(float64(n)) }

// Worker retry/DLQ metrics
func (r *Registry) IncRetry() { r.retryTotal.Inc() }
func (r *Registry) IncDLQ()   { r.dlqTotal.Inc() }
