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
    });
    let res = messaging_server::run_server(cfg).await;
    assert!(res.is_err(), "expected bind error, got: {:?}", res);
    let err = res.err().unwrap();
    assert!(err.contains("failed to bind"), "error: {}", err);
}
