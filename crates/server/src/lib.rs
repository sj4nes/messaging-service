use axum::{routing::get, Json, Router};
use messaging_core::Config;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

pub fn build_router(health_path: &str) -> Router {
    let path = health_path.to_string();
    Router::new().route(&path, get(health_handler))
}

async fn health_handler() -> Json<Health> {
    tracing::info!(target: "server", event = "health", "health check");
    Json(Health { status: "ok" })
}

pub async fn run_server(
    config: Arc<Config>,
) -> Result<(tokio::task::JoinHandle<()>, SocketAddr), String> {
    let router = build_router(&config.health_path);

    // Bind to 127.0.0.1 for tests; for production US1 scope, bind on 0.0.0.0
    let bind_addr: SocketAddr = ([0, 0, 0, 0], config.port).into();
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|e| format!("failed to bind: {e}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("failed to read local addr: {e}"))?;

    tracing::info!(target: "server", event = "startup", %local_addr, health_path = %config.health_path, "listening");

    let server = axum::serve(listener, router.into_make_service());
    let handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("server error: {e}");
        }
    });
    Ok((handle, local_addr))
}
