// Integration test: incoming webhooks for both providers (SMS/MMS and Email)
use messaging_core::Config;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn webhook_sms_and_email_are_accepted() {
    // Start server on an ephemeral port
    let cfg = Arc::new(Config {
        port: 0,
        health_path: "/health".to_string(),
        log_level: "info".to_string(),
    });
    let (handle, addr) = messaging_server::run_server(cfg)
        .await
        .expect("server start");

    let client = reqwest::Client::new();
    let base = format!("http://{}", addr);

    // Prepare webhook SMS payload (type=sms)
    let sms_payload = serde_json::json!({
        "from": "+15550001",
        "to": "+15550002",
        "type": "sms",
        "messaging_provider_id": "prov-msg-1",
        "body": "inbound from provider",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    // Prepare webhook Email payload
    let email_payload = serde_json::json!({
        "from": "a@example.com",
        "to": "b@example.com",
        "xillio_id": "email-123",
        "body": "inbound email from provider",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    // POST SMS webhook
    let sms_resp = client
        .post(format!("{}/api/webhooks/sms", base))
        .json(&sms_payload)
        .send()
        .await
        .expect("sms webhook send");
    assert!(
        sms_resp.status().is_success(),
        "expected 2xx for SMS webhook, got {}",
        sms_resp.status()
    );
    let sms_json: serde_json::Value = sms_resp.json().await.expect("sms webhook json");
    assert_eq!(sms_json.get("status"), Some(&serde_json::json!("accepted")));

    // POST Email webhook
    let email_resp = client
        .post(format!("{}/api/webhooks/email", base))
        .json(&email_payload)
        .send()
        .await
        .expect("email webhook send");
    assert!(
        email_resp.status().is_success(),
        "expected 2xx for Email webhook, got {}",
        email_resp.status()
    );
    let email_json: serde_json::Value = email_resp.json().await.expect("email webhook json");
    assert_eq!(
        email_json.get("status"),
        Some(&serde_json::json!("accepted"))
    );

    // Allow brief time for any async enqueue paths (in-memory) to complete without DB
    sleep(Duration::from_millis(50)).await;

    // Shutdown server
    handle.abort();
}
