use tracing_subscriber::{fmt, EnvFilter};

/// Initialize global logging using tracing-subscriber.
///
/// Notes on sensitive data:
/// - Do NOT log secrets (API keys, passwords, tokens). When logging config or headers, either omit
///   the field entirely or redact it using a helper like `redact_secret`.
/// - Prefer structured fields over interpolated strings so filters/redactors can work reliably.
/// - Future: consider a JSON formatter and a layer for automatic redaction of well-known keys.

pub fn init_logging(level: &str) -> Result<(), String> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .map_err(|e| format!("invalid log level or filter: {e}"))?;

    // try_init returns Err if a global subscriber is already set; treat as Ok for idempotence
    let _ = fmt().with_env_filter(filter).try_init();
    Ok(())
}

/// Minimal redaction helper. Use for values that must never appear in logs.
pub fn redact_secret<T>(_value: T) -> &'static str {
    "***"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_is_idempotent() {
        assert!(init_logging("info").is_ok());
        // second init shouldn't fail
        assert!(init_logging("debug").is_ok());
    }
}
