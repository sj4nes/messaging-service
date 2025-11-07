use crate::config::ApiConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    RateLimited,
    Error,
    Timeout,
}

fn clamp_prob(p: i32) -> u32 {
    if p < 0 {
        0
    } else if p > 100 {
        100
    } else {
        p as u32
    }
}

/// Pick an outcome based on configured probabilities. Success is whatever remains.
pub fn pick_outcome(cfg: &ApiConfig) -> Outcome {
    let timeout = clamp_prob(cfg.provider_timeout_pct as i32);
    let error = clamp_prob(cfg.provider_error_pct as i32);
    let ratelimit = clamp_prob(cfg.provider_ratelimit_pct as i32);
    let roll = next_roll(cfg.provider_seed);
    // Map roll 0..=99 into buckets
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
    CELL.get_or_init(|| AtomicU64::new(0x9E3779B97F4A7C15)) // Golden ratio seed fallback
}

fn next_roll(seed: Option<u64>) -> u32 {
    // If a deterministic seed is specified, initialize once; otherwise, mix in time
    let state = rng_state();
    if let Some(s) = seed {
        // One-time init to provided seed (idempotent effect once used)
        let _ = state.compare_exchange(0x9E3779B97F4A7C15, s, Ordering::SeqCst, Ordering::SeqCst);
    } else {
        // Mix in coarse time to vary sequence
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        state.fetch_xor(t.rotate_left(7), Ordering::Relaxed);
    }
    // LCG step
    let prev = state.load(Ordering::Relaxed);
    let next = prev.wrapping_mul(6364136223846793005).wrapping_add(1);
    state.store(next, Ordering::Relaxed);
    (next % 100) as u32
}
