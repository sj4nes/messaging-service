use tracing_subscriber::{fmt, EnvFilter};

pub fn init_logging(level: &str) -> Result<(), String> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .map_err(|e| format!("invalid log level or filter: {e}"))?;

    // try_init returns Err if a global subscriber is already set; treat as Ok for idempotence
    let _ = fmt().with_env_filter(filter).try_init();
    Ok(())
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
