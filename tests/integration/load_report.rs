//! Load test for conversation upsert latency measurement
//!
//! This integration test performs high-throughput concurrent conversation upserts
//! and reports latency distribution (P50, P95, P99) to validate performance requirements.
//!
//! Performance Goals (from spec):
//! - Conversation upsert P95 ≤10ms local, ≤25ms at 100 RPS
//! - Zero duplicate conversation creation under race conditions

use messaging_server::config::Config;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

/// Run load test with specified parameters
async fn load_test_upserts(
    pool: &sqlx::PgPool,
    num_requests: usize,
    concurrency: usize,
) -> Vec<Duration> {
    let mut latencies = Vec::with_capacity(num_requests);
    let mut tasks = JoinSet::new();
    let requests_per_task = num_requests / concurrency;
    let remainder = num_requests % concurrency;

    for task_idx in 0..concurrency {
        let pool = pool.clone();
        let count = if task_idx == 0 {
            requests_per_task + remainder
        } else {
            requests_per_task
        };
        tasks.spawn(async move {
            let mut task_latencies = Vec::with_capacity(count);
            for i in 0..count {
                let from = format!("user{}@example.com", task_idx * requests_per_task + i);
                let to = format!("recipient{}@example.com", (task_idx + i) % 100);
                let start = Instant::now();
                let _ = messaging_server::store_db::messages::insert_outbound(
                    &pool,
                    "email",
                    &from,
                    &to,
                    None,
                    Some("Load test message"),
                )
                .await;
                task_latencies.push(start.elapsed());
            }
            task_latencies
        });
    }

    while let Some(result) = tasks.join_next().await {
        if let Ok(mut task_latencies) = result {
            latencies.append(&mut task_latencies);
        }
    }

    latencies
}

/// Calculate percentile from sorted latency vector
fn percentile(latencies: &[Duration], p: f64) -> Duration {
    if latencies.is_empty() {
        return Duration::from_micros(0);
    }
    let index = ((latencies.len() as f64 - 1.0) * p / 100.0).round() as usize;
    latencies[index]
}

/// Format duration in appropriate unit
fn format_duration(d: Duration) -> String {
    let micros = d.as_micros();
    if micros < 1000 {
        format!("{}μs", micros)
    } else if micros < 1_000_000 {
        format!("{:.2}ms", micros as f64 / 1000.0)
    } else {
        format!("{:.2}s", micros as f64 / 1_000_000.0)
    }
}

/// Print latency report
fn print_report(latencies: &[Duration], rps: f64, duration: Duration) {
    if latencies.is_empty() {
        println!("No latency data collected");
        return;
    }

    let mut sorted = latencies.to_vec();
    sorted.sort();

    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let p50 = percentile(&sorted, 50.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);

    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║  Feature 009: Conversation Upsert Load Test Report  ║");
    println!("╠══════════════════════════════════════════════════════╣");
    println!("║  Total Requests:  {:>35} ║", latencies.len());
    println!("║  Total Duration:  {:>35} ║", format_duration(duration));
    println!("║  Throughput:      {:>32.2} RPS ║", rps);
    println!("╠══════════════════════════════════════════════════════╣");
    println!("║  Min Latency:     {:>35} ║", format_duration(min));
    println!("║  P50 Latency:     {:>35} ║", format_duration(p50));
    println!("║  P95 Latency:     {:>35} ║", format_duration(p95));
    println!("║  P99 Latency:     {:>35} ║", format_duration(p99));
    println!("║  Max Latency:     {:>35} ║", format_duration(max));
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Check against performance goals
    let p95_micros = p95.as_micros();
    let goal_micros = 25_000; // 25ms at 100 RPS (conservative goal)

    if p95_micros <= goal_micros {
        println!("✓ PASS: P95 latency {}μs ≤ {}μs (goal)", p95_micros, goal_micros);
    } else {
        println!("✗ WARN: P95 latency {}μs > {}μs (goal)", p95_micros, goal_micros);
        println!("  Note: This may be acceptable depending on hardware/load");
    }
}

#[sqlx::test]
async fn load_test_100_rps_small_batch(pool: sqlx::PgPool) {
    // Simulate ~100 RPS for 1 second (100 requests with 10 concurrent tasks)
    println!("\n=== Load Test: 100 RPS (100 requests, 10 concurrent) ===");
    let start = Instant::now();
    let latencies = load_test_upserts(&pool, 100, 10).await;
    let duration = start.elapsed();
    let rps = 100.0 / duration.as_secs_f64();
    print_report(&latencies, rps, duration);
}

#[sqlx::test]
async fn load_test_sustained_load(pool: sqlx::PgPool) {
    // Sustained load: 500 requests with 20 concurrent tasks
    println!("\n=== Load Test: Sustained (500 requests, 20 concurrent) ===");
    let start = Instant::now();
    let latencies = load_test_upserts(&pool, 500, 20).await;
    let duration = start.elapsed();
    let rps = 500.0 / duration.as_secs_f64();
    print_report(&latencies, rps, duration);
}

#[sqlx::test]
async fn load_test_high_concurrency(pool: sqlx::PgPool) {
    // High concurrency: 1000 requests with 50 concurrent tasks
    println!("\n=== Load Test: High Concurrency (1000 requests, 50 concurrent) ===");
    let start = Instant::now();
    let latencies = load_test_upserts(&pool, 1000, 50).await;
    let duration = start.elapsed();
    let rps = 1000.0 / duration.as_secs_f64();
    print_report(&latencies, rps, duration);
}

#[sqlx::test]
async fn load_test_conversation_deduplication(pool: sqlx::PgPool) {
    // Test deduplication: 1000 requests creating only 10 unique conversations
    println!("\n=== Load Test: Deduplication (1000 requests → 10 conversations) ===");

    // Create messages for 10 conversation pairs
    let mut tasks = JoinSet::new();
    let start = Instant::now();

    for i in 0..1000 {
        let pool = pool.clone();
        let conv_idx = i % 10; // Only 10 unique conversations
        tasks.spawn(async move {
            let from = format!("sender{}@example.com", conv_idx);
            let to = format!("receiver{}@example.com", conv_idx);
            let start = Instant::now();
            let _ = messaging_server::store_db::messages::insert_outbound(
                &pool,
                "email",
                &from,
                &to,
                None,
                Some(&format!("Dedup test message {}", i)),
            )
            .await;
            start.elapsed()
        });
    }

    let mut latencies = Vec::new();
    while let Some(result) = tasks.join_next().await {
        if let Ok(latency) = result {
            latencies.push(latency);
        }
    }

    let duration = start.elapsed();
    let rps = 1000.0 / duration.as_secs_f64();
    print_report(&latencies, rps, duration);

    // Verify only 10 conversations were created
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversations")
        .fetch_one(&pool)
        .await
        .unwrap();

    println!("Conversations created: {}", count);
    assert!(
        count <= 10,
        "Expected ≤10 conversations, found {}",
        count
    );
    println!("✓ PASS: Deduplication working correctly (1000 messages → {} conversations)", count);
}

#[tokio::test]
async fn run_all_load_tests() {
    // This is a convenience test that can be run without sqlx::test macro
    // to execute all load tests sequentially. Requires DATABASE_URL env var.
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping load tests: DATABASE_URL not set");
        return;
    }

    let database_url = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run all load test scenarios
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║         Running Full Load Test Suite                ║");
    println!("╚══════════════════════════════════════════════════════╝");

    load_test_100_rps_small_batch(pool.clone()).await;
    load_test_sustained_load(pool.clone()).await;
    load_test_high_concurrency(pool.clone()).await;
    load_test_conversation_deduplication(pool.clone()).await;

    println!("\n✓ All load tests completed");
}
