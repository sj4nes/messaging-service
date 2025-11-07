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
    }
}
