use messaging_core::config::{ConfigSources, Source};
use messaging_core::{logging, Config};
use std::sync::Arc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cfg, sources) = Config::load_with_sources().map_err(|e| format!("config error: {e}"))?;
    // Initialize logging based on resolved level
    logging::init_logging(&cfg.log_level).map_err(|e| format!("logging init error: {e}"))?;

    // Log resolved config and detected sources
    log_config(&cfg, &sources);

    let cfg = Arc::new(cfg);
    let (handle, _addr) = messaging_server::run_server(cfg)
        .await
        .map_err(|e| format!("{e}"))?;
    // Block on the server task (graceful shutdown to be added in US3)
    let _ = handle.await;
    Ok(())
}

fn log_config(cfg: &Config, sources: &ConfigSources) {
    let src = |s: Source| match s {
        Source::Env => "env",
        Source::Dotenv => ".env",
        Source::Default => "default",
    };
    tracing::info!(
        target: "server",
        event = "config",
        port = cfg.port,
        port_source = %src(sources.port),
        health_path = %cfg.health_path,
        health_path_source = %src(sources.health_path),
        log_level = %cfg.log_level,
        log_level_source = %src(sources.log_level),
        "resolved configuration"
    );
}
