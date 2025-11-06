use serde::{Deserialize, Serialize};

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
        }
    }
}
