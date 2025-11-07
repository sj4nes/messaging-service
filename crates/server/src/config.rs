use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// API-specific configuration overlays (rates, sizes, breaker thresholds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Maximum JSON request body size in bytes (e.g., 256 KB)
    pub max_body_bytes: usize,
    /// Max attachments per message
    pub max_attachments: usize,
    /// Per-IP requests per minute (rolling/windowed depending on implementation)
    pub rate_limit_per_ip_per_min: u32,
    /// Per-sender requests per minute
    pub rate_limit_per_sender_per_min: u32,
    /// Circuit breaker: consecutive error threshold to open
    pub breaker_error_threshold: u32,
    /// Circuit breaker: open state duration in seconds before half-open
    pub breaker_open_secs: u64,
    /// Mock provider: percentage of timeouts (0-100)
    pub provider_timeout_pct: u32,
    /// Mock provider: percentage of 5xx errors (0-100)
    pub provider_error_pct: u32,
    /// Mock provider: percentage of 429 rate limits (0-100)
    pub provider_ratelimit_pct: u32,
    /// Mock provider: deterministic RNG seed (optional)
    pub provider_seed: Option<u64>,
    /// Worker: number of inbound events claimed per cycle
    pub worker_batch_size: u32,
    /// Worker: seconds before a claim is considered stale and can be reaped
    pub worker_claim_timeout_secs: u64,
    /// Worker: maximum processing retries before dead_letter
    pub worker_max_retries: u32,
    /// Worker: base backoff in milliseconds for exponential retry
    pub worker_backoff_base_ms: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            max_body_bytes: 256 * 1024,
            max_attachments: 8,
            rate_limit_per_ip_per_min: 120,
            rate_limit_per_sender_per_min: 60,
            breaker_error_threshold: 20,
            breaker_open_secs: 30,
            provider_timeout_pct: 0,
            provider_error_pct: 0,
            provider_ratelimit_pct: 0,
            provider_seed: None,
            worker_batch_size: 10,
            worker_claim_timeout_secs: 60,
            worker_max_retries: 5,
            worker_backoff_base_ms: 500,
        }
    }
}

impl ApiConfig {
    /// Load configuration from default.toml file and environment overrides (API_* vars).
    pub fn load() -> Self {
        // Allow specifying a custom config file via env
        if let Ok(path) = std::env::var("API_CONFIG_FILE") {
            let file_cfg = Self::from_file(path);
            return Self::apply_env_overrides(file_cfg);
        }
        let file_cfg = Self::from_file("crates/server/config/default.toml");
        Self::apply_env_overrides(file_cfg)
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref();
        match fs::read_to_string(p) {
            Ok(contents) => toml::from_str::<ApiConfig>(&contents).unwrap_or_else(|e| {
                tracing::warn!(target="server", path=%p.display(), error=%e, "Failed to parse default.toml; using built-in defaults");
                ApiConfig::default()
            }),
            Err(_) => {
                tracing::debug!(target="server", path=%p.display(), "default.toml not found; using built-in defaults");
                ApiConfig::default()
            }
        }
    }

    fn apply_env_overrides(mut cfg: Self) -> Self {
        // Helper macro to parse unsigned integers
        macro_rules! override_u {
            ($field:ident, $env:literal, $ty:ty) => {
                if let Ok(val) = std::env::var($env) {
                    if let Ok(parsed) = val.parse::<$ty>() {
                        cfg.$field = parsed as _;
                    } else {
                        tracing::warn!(target="server", key=$env, value=%val, "Invalid numeric env override");
                    }
                }
            };
        }
        override_u!(max_body_bytes, "API_MAX_BODY_BYTES", usize);
        override_u!(max_attachments, "API_MAX_ATTACHMENTS", u32);
        override_u!(
            rate_limit_per_ip_per_min,
            "API_RATE_LIMIT_PER_IP_PER_MIN",
            u32
        );
        override_u!(
            rate_limit_per_sender_per_min,
            "API_RATE_LIMIT_PER_SENDER_PER_MIN",
            u32
        );
        override_u!(breaker_error_threshold, "API_BREAKER_ERROR_THRESHOLD", u32);
        override_u!(breaker_open_secs, "API_BREAKER_OPEN_SECS", u64);
        override_u!(provider_timeout_pct, "API_PROVIDER_TIMEOUT_PCT", u32);
        override_u!(provider_error_pct, "API_PROVIDER_ERROR_PCT", u32);
        override_u!(provider_ratelimit_pct, "API_PROVIDER_RATELIMIT_PCT", u32);
        override_u!(worker_batch_size, "API_WORKER_BATCH_SIZE", u32);
        override_u!(
            worker_claim_timeout_secs,
            "API_WORKER_CLAIM_TIMEOUT_SECS",
            u64
        );
        override_u!(worker_max_retries, "API_WORKER_MAX_RETRIES", u32);
        override_u!(worker_backoff_base_ms, "API_WORKER_BACKOFF_BASE_MS", u64);
        if let Ok(seed) = std::env::var("API_PROVIDER_SEED") {
            match seed.parse::<u64>() {
                Ok(n) => cfg.provider_seed = Some(n),
                Err(_) => {
                    tracing::warn!(target="server", key="API_PROVIDER_SEED", value=%seed, "Invalid numeric env override")
                }
            }
        }
        cfg
    }
}
