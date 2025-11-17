use messaging_core::Config;
use messaging_server::run_server;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::test(flavor = "multi_thread")]
async fn health_ok() {
    let cfg = Config {
        port: 0,
        health_path: "/healthz".into(),
        log_level: "info".into(),
        conversation_snippet_length: 64,
        auth_session_expiry_min: 30,
        rate_limit_per_ip_per_min: 120,
        rate_limit_per_sender_per_min: 60,
        argon2_memory_mb: 64,
        argon2_time_cost: 3,
        argon2_parallelism: 1,
        security_headers_enabled: true,
        csp_default_src: "'self'".into(),
        ssrf_allowlist: vec![],
    };
    let cfg = Arc::new(cfg);
    let (handle, addr) = run_server(cfg).await.expect("server start");

    let url = format!("http://{}{}", addr, "/healthz");
    let res = reqwest::get(&url).await.expect("request");
    assert!(res.status().is_success());
    let body: serde_json::Value = res.json().await.expect("json");
    assert_eq!(body["status"], "ok");

    handle.abort();
}

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

#[tokio::test(flavor = "multi_thread")]
async fn health_logs() {
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
    // Try set as global; if already set, fall back to scoped default
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

    // Start server
    let cfg = Config {
        port: 0,
        health_path: "/healthz".into(),
        log_level: "info".into(),
        conversation_snippet_length: 64,
        auth_session_expiry_min: 30,
        rate_limit_per_ip_per_min: 120,
        rate_limit_per_sender_per_min: 60,
        argon2_memory_mb: 64,
        argon2_time_cost: 3,
        argon2_parallelism: 1,
        security_headers_enabled: true,
        csp_default_src: "'self'".into(),
        ssrf_allowlist: vec![],
    };
    let cfg = Arc::new(cfg);
    let (handle, addr) = run_server(cfg).await.expect("server start");

    let url = format!("http://{}{}", addr, "/healthz");
    let _ = reqwest::get(&url).await.expect("request");

    // Read captured logs
    let captured = {
        let guard = buf.lock().unwrap();
        String::from_utf8_lossy(&guard).to_string()
    };
    assert!(
        captured.contains("startup") && captured.contains("health"),
        "logs: {}",
        captured
    );

    handle.abort();
}
