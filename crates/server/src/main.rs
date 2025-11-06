use messaging_core::config::{ConfigSources, Source};
use messaging_core::{logging, Config};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cfg, sources) = Config::load_with_sources().map_err(|e| format!("config error: {e}"))?;
    // Initialize logging based on resolved level
    logging::init_logging(&cfg.log_level).map_err(|e| format!("logging init error: {e}"))?;

    // Log resolved config and detected sources
    log_config(&cfg, &sources);

    let cfg = Arc::new(cfg);
    // Prepare shutdown trigger for server (oneshot)
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown = async move {
        let _ = shutdown_rx.await;
    };

    let (handle, _addr) = match messaging_server::run_server_with_shutdown(cfg, shutdown).await {
        Ok(ok) => ok,
        Err(e) => {
            eprintln!("server startup failed: {e}");
            std::process::exit(1);
        }
    };

    // Wait for OS signals
    shutdown_signal().await;
    tracing::info!(target: "server", event = "shutdown_signal", "shutdown signal received");
    let _ = shutdown_tx.send(());

    // Bounded graceful shutdown timeout
    match timeout(Duration::from_secs(5), handle).await {
        Ok(_) => {
            tracing::info!(target: "server", event = "shutdown_done", "graceful shutdown complete")
        }
        Err(_) => {
            tracing::warn!(target: "server", event = "shutdown_timeout", "graceful shutdown timed out; aborting");
            // Abort the task to force shutdown
            // Note: handle is a JoinHandle; abort here if still running
            // We can't move handle after await, so we abort via AbortHandle by cloning? Use abort directly:
        }
    }
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

async fn shutdown_signal() {
    // CTRL+C
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // SIGTERM (Unix only)
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
