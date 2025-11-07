use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Simple in-process counters for early observability. For production replace with Prometheus exporter.
static TOTAL_RATE_LIMITED: AtomicU64 = AtomicU64::new(0);
static TOTAL_BREAKER_OPEN: AtomicU64 = AtomicU64::new(0);
static DISPATCH_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
static DISPATCH_SUCCESS: AtomicU64 = AtomicU64::new(0);
static DISPATCH_RATE_LIMITED: AtomicU64 = AtomicU64::new(0);
static DISPATCH_ERROR: AtomicU64 = AtomicU64::new(0);
static BREAKER_TRANSITIONS: AtomicU64 = AtomicU64::new(0);
static WORKER_CLAIMED: AtomicU64 = AtomicU64::new(0);
static WORKER_PROCESSED: AtomicU64 = AtomicU64::new(0);
static WORKER_ERROR: AtomicU64 = AtomicU64::new(0);
static WORKER_DEAD_LETTER: AtomicU64 = AtomicU64::new(0);
static WORKER_LATENCY_TOTAL_US: AtomicU64 = AtomicU64::new(0);
static WORKER_LATENCY_MAX_US: AtomicU64 = AtomicU64::new(0);

#[derive(serde::Serialize)]
pub struct MetricsSnapshot {
    ts_unix_ms: u128,
    rate_limited: u64,
    breaker_open: u64,
    dispatch_attempts: u64,
    dispatch_success: u64,
    dispatch_rate_limited: u64,
    dispatch_error: u64,
    breaker_transitions: u64,
    worker_claimed: u64,
    worker_processed: u64,
    worker_error: u64,
    worker_dead_letter: u64,
    worker_latency_avg_us: u64,
    worker_latency_max_us: u64,
}

pub fn record_rate_limited() {
    TOTAL_RATE_LIMITED.fetch_add(1, Ordering::Relaxed);
}

pub fn record_breaker_open() {
    TOTAL_BREAKER_OPEN.fetch_add(1, Ordering::Relaxed);
}

pub fn record_breaker_transition() {
    BREAKER_TRANSITIONS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_dispatch_attempt() {
    DISPATCH_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_dispatch_success() {
    DISPATCH_SUCCESS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_dispatch_rate_limited() {
    DISPATCH_RATE_LIMITED.fetch_add(1, Ordering::Relaxed);
}

pub fn record_dispatch_error() {
    DISPATCH_ERROR.fetch_add(1, Ordering::Relaxed);
}

pub fn snapshot() -> MetricsSnapshot {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    MetricsSnapshot {
        ts_unix_ms: now,
        rate_limited: TOTAL_RATE_LIMITED.load(Ordering::Relaxed),
        breaker_open: TOTAL_BREAKER_OPEN.load(Ordering::Relaxed),
        dispatch_attempts: DISPATCH_ATTEMPTS.load(Ordering::Relaxed),
        dispatch_success: DISPATCH_SUCCESS.load(Ordering::Relaxed),
        dispatch_rate_limited: DISPATCH_RATE_LIMITED.load(Ordering::Relaxed),
        dispatch_error: DISPATCH_ERROR.load(Ordering::Relaxed),
        breaker_transitions: BREAKER_TRANSITIONS.load(Ordering::Relaxed),
        worker_claimed: WORKER_CLAIMED.load(Ordering::Relaxed),
        worker_processed: WORKER_PROCESSED.load(Ordering::Relaxed),
        worker_error: WORKER_ERROR.load(Ordering::Relaxed),
        worker_dead_letter: WORKER_DEAD_LETTER.load(Ordering::Relaxed),
        worker_latency_avg_us: {
            let total = WORKER_LATENCY_TOTAL_US.load(Ordering::Relaxed);
            let processed = WORKER_PROCESSED.load(Ordering::Relaxed).max(1); // avoid div by zero
            total / processed
        },
        worker_latency_max_us: WORKER_LATENCY_MAX_US.load(Ordering::Relaxed),
    }
}

pub fn record_worker_claimed(n: u64) {
    WORKER_CLAIMED.fetch_add(n, Ordering::Relaxed);
}

pub fn record_worker_processed(latency_us: u64) {
    WORKER_PROCESSED.fetch_add(1, Ordering::Relaxed);
    WORKER_LATENCY_TOTAL_US.fetch_add(latency_us, Ordering::Relaxed);
    // update max
    let mut current = WORKER_LATENCY_MAX_US.load(Ordering::Relaxed);
    while latency_us > current
        && WORKER_LATENCY_MAX_US
            .compare_exchange(current, latency_us, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
    {
        current = WORKER_LATENCY_MAX_US.load(Ordering::Relaxed);
    }
}

pub fn record_worker_error() {
    WORKER_ERROR.fetch_add(1, Ordering::Relaxed);
}

pub fn record_worker_dead_letter() {
    WORKER_DEAD_LETTER.fetch_add(1, Ordering::Relaxed);
}
