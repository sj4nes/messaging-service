// Feature 008 - US1 Integration test: SMS vs Email dispatch logs/metrics (T021)
use messaging_core::Config;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn sms_and_email_route_to_distinct_providers() {
    // Start server
    let cfg = Arc::new(Config {
        port: 0,
        health_path: "/health".to_string(),
        log_level: "info".to_string(),
    });
    let (handle, addr) = messaging_server::run_server(cfg.clone())
        .await
        .expect("server start");

    // Send SMS
    let sms_body = serde_json::json!({
        "from": "+15550001",
        "to": "+15550002",
        "type": "sms",
        "body": "hello sms",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let email_body = serde_json::json!({
        "from": "a@example.com",
        "to": "b@example.com",
        "body": "hello email",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "attachments": ["file.txt"],
    });

    let client = reqwest::Client::new();
    let base = format!("http://{}", addr);
    let sms_resp = client
        .post(format!("{}/api/messages/sms", base))
        .json(&sms_body)
        .send()
        .await
        .expect("sms send");
    assert!(sms_resp.status().is_success());
    let email_resp = client
        .post(format!("{}/api/messages/email", base))
        .json(&email_body)
        .send()
        .await
        .expect("email send");
    assert!(email_resp.status().is_success());

    // Allow worker to process events
    sleep(Duration::from_millis(150)).await;

    // Fetch metrics
    let metrics_resp = client
        .get(format!("{}/metrics", base))
        .send()
        .await
        .expect("metrics");
    assert!(metrics_resp.status().is_success());
    let snapshot: serde_json::Value = metrics_resp.json().await.expect("metrics json");

    // Validate provider counters incremented
    let sms_attempts = snapshot
        .get("provider_sms_mms_attempts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let email_attempts = snapshot
        .get("provider_email_attempts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(
        sms_attempts >= 1,
        "expected at least one sms/mms provider attempt"
    );
    assert!(
        email_attempts >= 1,
        "expected at least one email provider attempt"
    );

    // Shutdown server
    handle.abort();
}
