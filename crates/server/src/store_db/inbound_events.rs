use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

/// Insert inbound event idempotently using unique index on (channel, provider_message_id)
pub async fn insert_inbound_event(
    pool: &PgPool,
    channel: &str,
    from: &str,
    to: &str,
    provider_message_id: Option<&str>,
    payload: serde_json::Value,
) -> Result<()> {
    // attempts/status columns from existing schema: attempts -> attempt_count; status -> pending
    // Map to existing: status 'pending'
    sqlx::query!(
        r#"INSERT INTO inbound_events (event_type, payload, available_at, status, channel, "from", "to", provider_message_id)
            VALUES ($1, $2, now(), 'pending', $1, $3, $4, $5)
            ON CONFLICT (channel, provider_message_id) WHERE provider_message_id IS NOT NULL DO NOTHING"#,
        channel,
        payload,
        from,
        to,
        provider_message_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Claim a batch of pending events; set status=processing and return rows
pub async fn claim_batch(pool: &PgPool, batch_size: i64) -> Result<Vec<i64>> {
    // Use SKIP LOCKED pattern
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;
    let rows = sqlx::query!(
        r#"SELECT id FROM inbound_events
            WHERE status = 'pending' AND available_at <= now()
            ORDER BY id
            FOR UPDATE SKIP LOCKED LIMIT $1"#,
        batch_size
    )
    .fetch_all(&mut *tx)
    .await?;
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    if !ids.is_empty() {
        sqlx::query!(
            r#"UPDATE inbound_events SET status='processing', updated_at=now()
                WHERE id = ANY($1)"#,
            &ids
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(ids)
}

pub async fn mark_processed(pool: &PgPool, id: i64) -> Result<()> {
    sqlx::query!(
        r#"UPDATE inbound_events SET status='done', processed_at=now(), updated_at=now() WHERE id=$1"#,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetch a single inbound event payload and metadata for processing
pub async fn fetch_event(
    pool: &PgPool,
    id: i64,
) -> Result<Option<(String, Option<String>, Option<String>, serde_json::Value)>> {
    let row = sqlx::query!(
        r#"SELECT channel, "from", "to", payload FROM inbound_events WHERE id=$1"#,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.channel.unwrap_or_default(), r.from, r.to, r.payload)))
}

pub async fn mark_error(
    pool: &PgPool,
    id: i64,
    code: &str,
    message: &str,
    max_retries: i32,
    backoff_base_ms: i64,
) -> Result<bool> {
    // Increment attempts; compute exponential backoff; set available_at for retry; if exceeds max, mark dead
    let rec = sqlx::query!(r#"SELECT attempts FROM inbound_events WHERE id=$1"#, id)
        .fetch_one(pool)
        .await?;
    let attempts = rec.attempts + 1;
    if attempts > max_retries {
        sqlx::query!(
            r#"UPDATE inbound_events SET status='dead', error_code=$2, error_message=$3, attempts=$4, updated_at=now() WHERE id=$1"#,
            id,
            code,
            message,
            attempts
        )
        .execute(pool)
        .await?;
        Ok(true)
    } else {
        // backoff = base_ms * 2^((attempts-1)) with a simple cap at 60s for now
        let pow = (attempts - 1).max(0) as u32;
        // Compute 2^pow as i64 with simple loop to avoid shifting on i64
        let mut factor: i64 = 1;
        for _ in 0..pow {
            factor = factor.saturating_mul(2);
        }
        let mut delay_ms: i64 = backoff_base_ms.saturating_mul(factor);
        if delay_ms > 60_000 {
            delay_ms = 60_000;
        }
        let delay_secs: i32 = ((delay_ms as f64) / 1000.0).round() as i32;
        sqlx::query!(
            r#"UPDATE inbound_events SET status='pending', error_code=$2, error_message=$3, attempts=$4,
                    available_at = (now() + make_interval(secs := $5::INT)), updated_at=now() WHERE id=$1"#,
            id,
            code,
            message,
            attempts,
            delay_secs
        )
        .execute(pool)
        .await?;
        Ok(false)
    }
}

/// Reap stale processing claims after timeout_secs
pub async fn reap_stale(pool: &PgPool, timeout_secs: i64) -> Result<u64> {
    let cutoff: DateTime<Utc> = Utc::now() - chrono::Duration::seconds(timeout_secs);
    let res = sqlx::query!(
        r#"UPDATE inbound_events SET status='pending', updated_at=now()
            WHERE status='processing' AND updated_at < $1"#,
        cutoff
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}
