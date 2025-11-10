// Feature 008 - US2 Integration test (T027): Per-provider circuit breaker isolation
use messaging_core::Config;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn sms_breaker_opens_while_email_remains_closed() {
    // Configure environment: force SMS/MMS errors, email succeeds, low threshold
    std::env::set_var("API_BREAKER_ERROR_THRESHOLD", "1");
    std::env::set_var("API_PROVIDER_SMS_ERROR_PCT", "100");
    std::env::set_var("API_PROVIDER_EMAIL_ERROR_PCT", "0");
    std::env::set_var("API_PROVIDER_SMS_SEED", "42");
    std::env::set_var("API_PROVIDER_EMAIL_SEED", "1337");

    let cfg = Arc::new(Config {
        port: 0,
        health_path: "/health".to_string(),
        log_level: "info".to_string(),
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
    let (handle, addr) = messaging_server::run_server(cfg.clone())
        .await
        .expect("server start");

    let client = reqwest::Client::new();
    let base = format!("http://{}", addr);

    // Send one SMS (expected error -> breaker opens)
    let sms_body = serde_json::json!({
        "from": "+15550001",
        "to": "+15550002",
        "type": "sms",
        "body": "trigger error",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let sms_resp = client
        .post(format!("{}/api/messages/sms", base))
        .json(&sms_body)
        .send()
        .await
        .expect("sms send");
    assert!(sms_resp.status().is_success());

    // Send one Email (should succeed, breaker stays closed)
    let email_body = serde_json::json!({
        "from": "a@example.com",
        "to": "b@example.com",
        "body": "ok email",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let email_resp = client
        .post(format!("{}/api/messages/email", base))
        .json(&email_body)
        .send()
        .await
        .expect("email send");
    assert!(email_resp.status().is_success());

    // Allow worker to process events
    sleep(Duration::from_millis(150)).await;

    // Send second SMS which should be short-circuited by open breaker (still 200 at API layer)
    let sms_resp2 = client
        .post(format!("{}/api/messages/sms", base))
        .json(&sms_body)
        .send()
        .await
        .expect("sms send 2");
    assert!(sms_resp2.status().is_success());

    // Fetch metrics snapshot
    let metrics_resp = client
        .get(format!("{}/metrics", base))
        .send()
        .await
        .expect("metrics");
    assert!(metrics_resp.status().is_success());
    let snapshot: serde_json::Value = metrics_resp.json().await.expect("metrics json");

    // Validate per-provider breaker transitions: sms >=1, email ==0
    let sms_transitions = snapshot
        .get("provider_sms_mms_breaker_transitions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let email_transitions = snapshot
        .get("provider_email_breaker_transitions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(
        sms_transitions >= 1,
        "expected at least one sms/mms breaker transition (open)"
    );
    assert_eq!(email_transitions, 0, "email breaker should not transition");

    // Global transition counter should exist (value may or may not change depending on global layer)
    let _global_transitions = snapshot
        .get("breaker_transitions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    handle.abort();
}
