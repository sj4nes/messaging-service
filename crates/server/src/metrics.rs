use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Simple in-process counters for early observability. For production replace with Prometheus exporter.
static TOTAL_RATE_LIMITED: AtomicU64 = AtomicU64::new(0);
static TOTAL_BREAKER_OPEN: AtomicU64 = AtomicU64::new(0);

#[derive(serde::Serialize)]
pub struct MetricsSnapshot {
    ts_unix_ms: u128,
    rate_limited: u64,
    breaker_open: u64,
}

pub fn record_rate_limited() {
    TOTAL_RATE_LIMITED.fetch_add(1, Ordering::Relaxed);
}

pub fn record_breaker_open() {
    TOTAL_BREAKER_OPEN.fetch_add(1, Ordering::Relaxed);
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
    }
}
