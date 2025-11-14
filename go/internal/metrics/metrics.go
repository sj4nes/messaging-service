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
	// Provider metrics
	providerAttempts      *prometheus.CounterVec
	providerSuccess       *prometheus.CounterVec
	providerRateLimited   *prometheus.CounterVec
	providerError         *prometheus.CounterVec
	invalidRouting           prometheus.Counter
	providerBreakerTransition prometheus.Counter
	providerBreakerOpen       prometheus.Counter
	started                  int64 // gauge-like via atomic load for quick introspection if needed
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

	// provider metrics
	providerAttempts := prometheus.NewCounterVec(prometheus.CounterOpts{Name: "provider_attempts_total", Help: "Provider attempt counts (labeled by provider)"}, []string{"provider"})
	providerSuccess := prometheus.NewCounterVec(prometheus.CounterOpts{Name: "provider_success_total", Help: "Provider success counts (labeled by provider)"}, []string{"provider"})
	providerRateLimited := prometheus.NewCounterVec(prometheus.CounterOpts{Name: "provider_rate_limited_total", Help: "Provider rate-limited counts (labeled by provider)"}, []string{"provider"})
	providerError := prometheus.NewCounterVec(prometheus.CounterOpts{Name: "provider_error_total", Help: "Provider error counts (labeled by provider)"}, []string{"provider"})
	inv := prometheus.NewCounter(prometheus.CounterOpts{Name: "invalid_routing", Help: "Invalid routing attempts for outbound channel"})
	_ = reg.Register(providerAttempts)
	_ = reg.Register(providerSuccess)
	_ = reg.Register(providerRateLimited)
	_ = reg.Register(providerError)
	_ = reg.Register(inv)
	btrans := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_breaker_transition_total", Help: "Per-provider circuit breaker transitions"})
	bopen := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_breaker_open_total", Help: "Count of short-circuited requests for provider (breaker open)"})
	_ = reg.Register(btrans)
	_ = reg.Register(bopen)
	r := &Registry{reg: reg, workerProcessed: wp, enqueueAttempt: enqA, enqueueSuccess: enqS, enqueueFailure: enqF, queueDepth: qd, retryTotal: rty, dlqTotal: dlq, providerAttempts: providerAttempts, providerSuccess: providerSuccess, providerRateLimited: providerRateLimited, providerError: providerError, invalidRouting: inv, providerBreakerTransition: btrans, providerBreakerOpen: bopen}
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

func (r *Registry) RecordProviderAttempt(label string) {
	r.providerAttempts.WithLabelValues(label).Inc()
}

func (r *Registry) RecordProviderSuccess(label string) {
	r.providerSuccess.WithLabelValues(label).Inc()
}

func (r *Registry) RecordProviderRateLimited(label string) {
	r.providerRateLimited.WithLabelValues(label).Inc()
}

func (r *Registry) RecordProviderError(label string) {
	r.providerError.WithLabelValues(label).Inc()
}

func (r *Registry) RecordInvalidRouting() { r.invalidRouting.Inc() }

func (r *Registry) RecordProviderBreakerTransition(label string) {
	r.providerBreakerTransition.Inc()
	_ = label
}

func (r *Registry) RecordProviderBreakerOpen(label string) {
	r.providerBreakerOpen.Inc()
	_ = label
}
