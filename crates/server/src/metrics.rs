use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Simple in-process counters for early observability. For production replace with Prometheus exporter.
static TOTAL_RATE_LIMITED: AtomicU64 = AtomicU64::new(0);
static TOTAL_BREAKER_OPEN: AtomicU64 = AtomicU64::new(0);
static DISPATCH_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
static DISPATCH_SUCCESS: AtomicU64 = AtomicU64::new(0);
static DISPATCH_RATE_LIMITED: AtomicU64 = AtomicU64::new(0);
static DISPATCH_ERROR: AtomicU64 = AtomicU64::new(0);
// Per-provider counters (Feature 008 US1)
static SMS_MMS_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
static SMS_MMS_SUCCESS: AtomicU64 = AtomicU64::new(0);
static SMS_MMS_RATE_LIMITED: AtomicU64 = AtomicU64::new(0);
static SMS_MMS_ERROR: AtomicU64 = AtomicU64::new(0);
static EMAIL_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
static EMAIL_SUCCESS: AtomicU64 = AtomicU64::new(0);
static EMAIL_RATE_LIMITED: AtomicU64 = AtomicU64::new(0);
static EMAIL_ERROR: AtomicU64 = AtomicU64::new(0);
static BREAKER_TRANSITIONS: AtomicU64 = AtomicU64::new(0);
static WORKER_CLAIMED: AtomicU64 = AtomicU64::new(0);
static WORKER_PROCESSED: AtomicU64 = AtomicU64::new(0);
static WORKER_ERROR: AtomicU64 = AtomicU64::new(0);
static WORKER_DEAD_LETTER: AtomicU64 = AtomicU64::new(0);
static WORKER_LATENCY_TOTAL_US: AtomicU64 = AtomicU64::new(0);
static WORKER_LATENCY_MAX_US: AtomicU64 = AtomicU64::new(0);

// --- Feature 008 placeholders (Phase 1) ---
// Provider label constants used for per-provider metrics in later phases
pub const PROVIDER_LABEL_SMS_MMS: &str = "sms-mms";
pub const PROVIDER_LABEL_EMAIL: &str = "email";

#[derive(serde::Serialize)]
pub struct MetricsSnapshot {
    ts_unix_ms: u128,
    rate_limited: u64,
    breaker_open: u64,
    dispatch_attempts: u64,
    dispatch_success: u64,
    dispatch_rate_limited: u64,
    dispatch_error: u64,
    provider_sms_mms_attempts: u64,
    provider_sms_mms_success: u64,
    provider_sms_mms_rate_limited: u64,
    provider_sms_mms_error: u64,
    provider_email_attempts: u64,
    provider_email_success: u64,
    provider_email_rate_limited: u64,
    provider_email_error: u64,
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

pub fn record_provider_attempt(label: &str) {
    match label {
        PROVIDER_LABEL_SMS_MMS => {
            SMS_MMS_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
        }
        PROVIDER_LABEL_EMAIL => {
            EMAIL_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

pub fn record_provider_success(label: &str) {
    match label {
        PROVIDER_LABEL_SMS_MMS => {
            SMS_MMS_SUCCESS.fetch_add(1, Ordering::Relaxed);
        }
        PROVIDER_LABEL_EMAIL => {
            EMAIL_SUCCESS.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

pub fn record_provider_rate_limited(label: &str) {
    match label {
        PROVIDER_LABEL_SMS_MMS => {
            SMS_MMS_RATE_LIMITED.fetch_add(1, Ordering::Relaxed);
        }
        PROVIDER_LABEL_EMAIL => {
            EMAIL_RATE_LIMITED.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

pub fn record_provider_error(label: &str) {
    match label {
        PROVIDER_LABEL_SMS_MMS => {
            SMS_MMS_ERROR.fetch_add(1, Ordering::Relaxed);
        }
        PROVIDER_LABEL_EMAIL => {
            EMAIL_ERROR.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
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
        provider_sms_mms_attempts: SMS_MMS_ATTEMPTS.load(Ordering::Relaxed),
        provider_sms_mms_success: SMS_MMS_SUCCESS.load(Ordering::Relaxed),
        provider_sms_mms_rate_limited: SMS_MMS_RATE_LIMITED.load(Ordering::Relaxed),
        provider_sms_mms_error: SMS_MMS_ERROR.load(Ordering::Relaxed),
        provider_email_attempts: EMAIL_ATTEMPTS.load(Ordering::Relaxed),
        provider_email_success: EMAIL_SUCCESS.load(Ordering::Relaxed),
        provider_email_rate_limited: EMAIL_RATE_LIMITED.load(Ordering::Relaxed),
        provider_email_error: EMAIL_ERROR.load(Ordering::Relaxed),
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

// --- Feature 008 additional metrics (Phase 2 placeholder impl) ---
static INVALID_ROUTING: AtomicU64 = AtomicU64::new(0);

pub fn record_invalid_routing() {
    INVALID_ROUTING.fetch_add(1, Ordering::Relaxed);
}
