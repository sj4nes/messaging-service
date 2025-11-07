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
/// falling back to global API_* values. Returns (Outcome, debug_roll) to support deterministic tests (US3).
pub fn pick_outcome_for_provider(provider: &str, cfg: &ApiConfig) -> (Outcome, u32) {
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
    let roll = next_roll(provider, seed);
    let outcome = if roll < timeout {
        Outcome::Timeout
    } else if roll < timeout + error {
        Outcome::Error
    } else if roll < timeout + error + ratelimit {
        Outcome::RateLimited
    } else {
        Outcome::Success
    };
    (outcome, roll)
}

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

const INIT: u64 = 0x9E3779B97F4A7C15;

fn rng_state_for(provider: &str) -> &'static AtomicU64 {
    match provider {
        "sms-mms" => {
            static SMS: OnceLock<AtomicU64> = OnceLock::new();
            SMS.get_or_init(|| AtomicU64::new(INIT))
        }
        "email" => {
            static EMAIL: OnceLock<AtomicU64> = OnceLock::new();
            EMAIL.get_or_init(|| AtomicU64::new(INIT))
        }
        _ => {
            static DEFAULT: OnceLock<AtomicU64> = OnceLock::new();
            DEFAULT.get_or_init(|| AtomicU64::new(INIT))
        }
    }
}

/// Deterministic pseudo-random roll scoped per provider. When a seed is provided, we set the
/// provider's RNG state once if it's still at INIT; subsequent calls advance a LCG.
fn next_roll(provider: &str, seed: Option<u64>) -> u32 {
    let state = rng_state_for(provider);
    if let Some(s) = seed {
        let _ = state.compare_exchange(INIT, s, Ordering::SeqCst, Ordering::SeqCst);
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

/// Forcefully seed a provider's RNG state (used at startup and in tests for determinism).
pub fn seed_provider_rng(provider: &str, seed: u64) {
    let state = rng_state_for(provider);
    state.store(seed, Ordering::Relaxed);
}

/// Initialize RNG seeds from ApiConfig at startup for deterministic sequences (US3 T030/T033).
pub fn init_rng_seeds(cfg: &ApiConfig) {
    if let Some(s) = cfg.provider_sms_seed.or(cfg.provider_seed) {
        seed_provider_rng("sms-mms", s);
    }
    if let Some(s) = cfg.provider_email_seed.or(cfg.provider_seed) {
        seed_provider_rng("email", s);
    }
}

/// Pure LCG step used for offline prediction without touching global state.
fn lcg_step(prev: u64) -> u64 {
    prev.wrapping_mul(6364136223846793005).wrapping_add(1)
}

/// Predict outcome counts for N rolls from a given seed without mutating global RNG.
pub fn predict_outcomes_from_seed(
    provider: &str,
    cfg: &ApiConfig,
    seed: u64,
    n: usize,
) -> (u32, u32, u32, u32) {
    let (timeout_pct, error_pct, ratelimit_pct, _) = match provider {
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
    let timeout = clamp(timeout_pct as i32);
    let error = clamp(error_pct as i32);
    let ratelimit = clamp(ratelimit_pct as i32);
    let mut state = seed;
    let mut succ = 0;
    let mut rate = 0;
    let mut err = 0;
    let mut to = 0;
    for _ in 0..n {
        state = lcg_step(state);
        let roll = (state % 100) as u32;
        if roll < timeout {
            to += 1;
        } else if roll < timeout + error {
            err += 1;
        } else if roll < timeout + error + ratelimit {
            rate += 1;
        } else {
            succ += 1;
        }
    }
    (succ, rate, err, to)
}
