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
	providerSmsAttempts       prometheus.Counter
	providerSmsSuccess        prometheus.Counter
	providerSmsRateLimited    prometheus.Counter
	providerSmsError          prometheus.Counter
	providerEmailAttempts     prometheus.Counter
	providerEmailSuccess      prometheus.Counter
	providerEmailRateLimited  prometheus.Counter
	providerEmailError        prometheus.Counter
	invalidRouting            prometheus.Counter
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

	// provider metrics
	psa := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_sms_mms_attempts", Help: "Per-provider attempts for sms/mms"})
	pss := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_sms_mms_success", Help: "Per-provider success for sms/mms"})
	psrl := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_sms_mms_rate_limited", Help: "Per-provider rate limit count for sms/mms"})
	pser := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_sms_mms_error", Help: "Per-provider errors for sms/mms"})
	pea := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_email_attempts", Help: "Per-provider attempts for email"})
	pes := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_email_success", Help: "Per-provider success for email"})
	perl := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_email_rate_limited", Help: "Per-provider rate limit count for email"})
	peer := prometheus.NewCounter(prometheus.CounterOpts{Name: "provider_email_error", Help: "Per-provider errors for email"})
	inv := prometheus.NewCounter(prometheus.CounterOpts{Name: "invalid_routing", Help: "Invalid routing attempts for outbound channel"})
	_ = reg.Register(psa)
	_ = reg.Register(pss)
	_ = reg.Register(psrl)
	_ = reg.Register(pser)
	_ = reg.Register(pea)
	_ = reg.Register(pes)
	_ = reg.Register(perl)
	_ = reg.Register(peer)
	_ = reg.Register(inv)
	r := &Registry{reg: reg, workerProcessed: wp, enqueueAttempt: enqA, enqueueSuccess: enqS, enqueueFailure: enqF, queueDepth: qd, retryTotal: rty, dlqTotal: dlq, providerSmsAttempts: psa, providerSmsSuccess: pss, providerSmsRateLimited: psrl, providerSmsError: pser, providerEmailAttempts: pea, providerEmailSuccess: pes, providerEmailRateLimited: perl, providerEmailError: peer, invalidRouting: inv}
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
	switch label {
	case "sms-mms":
		r.providerSmsAttempts.Inc()
	case "email":
		r.providerEmailAttempts.Inc()
	}
}

func (r *Registry) RecordProviderSuccess(label string) {
	switch label {
	case "sms-mms":
		r.providerSmsSuccess.Inc()
	case "email":
		r.providerEmailSuccess.Inc()
	}
}

func (r *Registry) RecordProviderRateLimited(label string) {
	switch label {
	case "sms-mms":
		r.providerSmsRateLimited.Inc()
	case "email":
		r.providerEmailRateLimited.Inc()
	}
}

func (r *Registry) RecordProviderError(label string) {
	switch label {
	case "sms-mms":
		r.providerSmsError.Inc()
	case "email":
		r.providerEmailError.Inc()
	}
}

func (r *Registry) RecordInvalidRouting() { r.invalidRouting.Inc() }
