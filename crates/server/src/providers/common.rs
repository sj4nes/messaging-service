//! Shared provider helpers (Feature 008)

use crate::config::ApiConfig;
use crate::providers::mock::Outcome;

fn clamp(p: i32) -> u32 {
    if p < 0 {
        0
    } else if p > 100 {
        100
    } else {
        p as u32
    }
}

/// Choose an outcome for a given logical provider based on per-provider overrides,
/// falling back to global API_* values.
pub fn pick_outcome_for_provider(provider: &str, cfg: &ApiConfig) -> Outcome {
    // Resolve effective pct/seed values based on provider-specific overrides
    let (timeout_pct, error_pct, ratelimit_pct, seed) = match provider {
        "sms-mms" => (
            cfg.provider_sms_timeout_pct
                .unwrap_or(cfg.provider_timeout_pct),
            cfg.provider_sms_error_pct.unwrap_or(cfg.provider_error_pct),
            cfg.provider_sms_ratelimit_pct
                .unwrap_or(cfg.provider_ratelimit_pct),
            cfg.provider_sms_seed.or(cfg.provider_seed),
        ),
        "email" => (
            cfg.provider_email_timeout_pct
                .unwrap_or(cfg.provider_timeout_pct),
            cfg.provider_email_error_pct
                .unwrap_or(cfg.provider_error_pct),
            cfg.provider_email_ratelimit_pct
                .unwrap_or(cfg.provider_ratelimit_pct),
            cfg.provider_email_seed.or(cfg.provider_seed),
        ),
        _ => (
            cfg.provider_timeout_pct,
            cfg.provider_error_pct,
            cfg.provider_ratelimit_pct,
            cfg.provider_seed,
        ),
    };

    // Compute a deterministic roll (0..99) given seed, similar to providers::mock
    let timeout = clamp(timeout_pct as i32);
    let error = clamp(error_pct as i32);
    let ratelimit = clamp(ratelimit_pct as i32);
    let roll = next_roll(seed);
    if roll < timeout {
        Outcome::Timeout
    } else if roll < timeout + error {
        Outcome::Error
    } else if roll < timeout + error + ratelimit {
        Outcome::RateLimited
    } else {
        Outcome::Success
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

fn rng_state() -> &'static AtomicU64 {
    static CELL: OnceLock<AtomicU64> = OnceLock::new();
    CELL.get_or_init(|| AtomicU64::new(0x9E3779B97F4A7C15))
}

fn next_roll(seed: Option<u64>) -> u32 {
    let state = rng_state();
    if let Some(s) = seed {
        let _ = state.compare_exchange(0x9E3779B97F4A7C15, s, Ordering::SeqCst, Ordering::SeqCst);
    } else {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        state.fetch_xor(t.rotate_left(7), Ordering::Relaxed);
    }
    let prev = state.load(Ordering::Relaxed);
    let next = prev.wrapping_mul(6364136223846793005).wrapping_add(1);
    state.store(next, Ordering::Relaxed);
    (next % 100) as u32
}
