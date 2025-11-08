use messaging_core::Config;
use messaging_server::run_server_with_shutdown;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tracing_subscriber::{fmt, EnvFilter};

struct Buf(Arc<Mutex<Vec<u8>>>);
impl Write for Buf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.0.lock().unwrap();
        guard.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn graceful_shutdown_logs() {
    // Capture logs
    let buf = Arc::new(Mutex::new(Vec::new()));
    let make_writer = {
        let buf = buf.clone();
        move || Buf(buf.clone())
    };
    let subscriber = fmt()
        .with_env_filter(EnvFilter::new("info"))
        .with_writer(make_writer)
        .finish();
    let _guard = match tracing::subscriber::set_global_default(subscriber) {
        Ok(()) => None,
        Err(_) => {
            let sub = fmt()
                .with_env_filter(EnvFilter::new("info"))
                .with_writer({
                    let b = buf.clone();
                    move || Buf(b.clone())
                })
                .finish();
            Some(tracing::subscriber::set_default(sub))
        }
    };

    // Setup shutdown trigger
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown = async move {
        let _ = rx.await;
    };

    // Start server
    let cfg = Config {
        port: 0,
        health_path: "/healthz".into(),
        log_level: "info".into(),
        conversation_snippet_length: 64,
    };
    let cfg = Arc::new(cfg);
    let (handle, _addr) = run_server_with_shutdown(cfg, shutdown)
        .await
        .expect("server start");

    // Trigger shutdown
    let _ = tx.send(());

    // Wait briefly for shutdown to complete
    let _ = handle.await;

    let captured = {
        let guard = buf.lock().unwrap();
        String::from_utf8_lossy(&guard).to_string()
    };
    assert!(captured.contains("shutdown"), "logs: {}", captured);
}
