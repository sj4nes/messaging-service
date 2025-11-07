// Feature 008 - US3 Deterministic integration test (T032)
// Starts server with fixed global seed and per-provider overrides; sends interleaved messages and
// asserts the outcome sequence is reproducible across two runs.
use messaging_core::Config;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};

static OUTCOMES_RUN1: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

async fn run_sequence_collect() -> Vec<String> {
    // Set deterministic seeds & provider error percentages
    std::env::set_var("API_PROVIDER_SEED", "777");
    std::env::set_var("API_PROVIDER_SMS_ERROR_PCT", "15");
    std::env::set_var("API_PROVIDER_EMAIL_ERROR_PCT", "20");
    std::env::set_var("API_PROVIDER_SMS_RATELIMIT_PCT", "5");
    std::env::set_var("API_PROVIDER_EMAIL_RATELIMIT_PCT", "10");

    let cfg = Arc::new(Config {
        port: 0,
        health_path: "/health".to_string(),
        log_level: "info".to_string(),
    });
    let (handle, addr) = messaging_server::run_server(cfg.clone())
        .await
        .expect("server start");
    let client = reqwest::Client::new();
    let base = format!("http://{}", addr);

    // Baseline snapshot
    let base_metrics_resp = client
        .get(format!("{}/metrics", base))
        .send()
        .await
        .expect("metrics base");
    assert!(base_metrics_resp.status().is_success());
    let base_snapshot: serde_json::Value =
        base_metrics_resp.json().await.expect("metrics json base");

    let mut outcomes = Vec::new();
    for i in 0..25 {
        // 50 total events (sms+email interleaved) processed by worker
        let sms_body = serde_json::json!({
            "from": "+1555SMS",
            "to": format!("+1555TO{}", i),
            "type": "sms",
            "body": format!("msg {}", i),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let email_body = serde_json::json!({
            "from": "a@example.com",
            "to": format!("b{}@example.com", i),
            "body": format!("email {}", i),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let sms_resp = client
            .post(format!("{}/api/messages/sms", base))
            .json(&sms_body)
            .send()
            .await
            .expect("sms");
        assert!(sms_resp.status().is_success());
        let email_resp = client
            .post(format!("{}/api/messages/email", base))
            .json(&email_body)
            .send()
            .await
            .expect("email");
        assert!(email_resp.status().is_success());
    }
    // Allow worker processing
    sleep(Duration::from_millis(400)).await;
    // Fetch metrics once (we rely on deterministic ordering of underlying LCG for outcome counts; for integration we capture counts only)
    let metrics_resp = client
        .get(format!("{}/metrics", base))
        .send()
        .await
        .expect("metrics");
    assert!(metrics_resp.status().is_success());
    let snapshot: serde_json::Value = metrics_resp.json().await.expect("metrics json");
    let sms_attempts = snapshot
        .get("provider_sms_mms_attempts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        - base_snapshot
            .get("provider_sms_mms_attempts")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
    let sms_errors = snapshot
        .get("provider_sms_mms_error")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        - base_snapshot
            .get("provider_sms_mms_error")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
    let sms_rl = snapshot
        .get("provider_sms_mms_rate_limited")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        - base_snapshot
            .get("provider_sms_mms_rate_limited")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
    let email_attempts = snapshot
        .get("provider_email_attempts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        - base_snapshot
            .get("provider_email_attempts")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
    let email_errors = snapshot
        .get("provider_email_error")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        - base_snapshot
            .get("provider_email_error")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
    let email_rl = snapshot
        .get("provider_email_rate_limited")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        - base_snapshot
            .get("provider_email_rate_limited")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
    outcomes.push(format!("sms_attempts:{}", sms_attempts));
    outcomes.push(format!("sms_errors:{}", sms_errors));
    outcomes.push(format!("sms_rl:{}", sms_rl));
    outcomes.push(format!("email_attempts:{}", email_attempts));
    outcomes.push(format!("email_errors:{}", email_errors));
    outcomes.push(format!("email_rl:{}", email_rl));

    handle.abort();
    outcomes
}

#[tokio::test]
async fn deterministic_mixed_traffic_counts_repeat() {
    // First run
    let run1 = run_sequence_collect().await;
    {
        let mut guard = OUTCOMES_RUN1.lock().unwrap();
        *guard = run1.clone();
    }
    // Clear env to avoid contamination then re-set with same values
    std::env::remove_var("API_PROVIDER_SEED");
    std::env::remove_var("API_PROVIDER_SMS_ERROR_PCT");
    std::env::remove_var("API_PROVIDER_EMAIL_ERROR_PCT");
    std::env::remove_var("API_PROVIDER_SMS_RATELIMIT_PCT");
    std::env::remove_var("API_PROVIDER_EMAIL_RATELIMIT_PCT");
    let run2 = run_sequence_collect().await;
    let guard = OUTCOMES_RUN1.lock().unwrap();
    assert_eq!(
        &run2, &*guard,
        "expected identical outcome metrics across deterministic runs"
    );
}
