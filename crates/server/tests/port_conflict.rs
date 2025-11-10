use messaging_core::Config;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

#[tokio::test]
async fn port_in_use_returns_error() {
    // Occupy a port
    let listener =
        TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 0))).expect("bind temp listener");
    let port = listener.local_addr().unwrap().port();

    // Attempt to start server with same port
    let cfg = Arc::new(Config {
        port,
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
    });
    let res = messaging_server::run_server(cfg).await;
    assert!(res.is_err(), "expected bind error, got: {:?}", res);
    let err = res.err().unwrap();
    assert!(err.contains("failed to bind"), "error: {}", err);
}
