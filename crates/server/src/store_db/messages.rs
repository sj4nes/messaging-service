use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;

use super::normalize::conversation_key;

/// Persist an inbound message using simplified bootstrap rules:
/// - Assumes a single bootstrap customer (id=1) and provider (id=1) already exist (future migration may ensure this)
/// - Upserts a conversation keyed by normalized endpoints+channel (temporary table-less approach: search existing messages for latest conversation id matching provider+participants)
#[instrument(skip(pool, body, attachments))]
pub async fn insert_from_inbound(
    pool: &PgPool,
    channel: &str,
    from: &str,
    to: &str,
    body: &str,
    attachments: &[String],
    timestamp: &str,
) -> Result<i64> {
    let conv_key = conversation_key(channel, from, to);
    // For now: ensure a conversation row exists (idempotent by topic conv_key) for bootstrap customer 1
    let convo_id = ensure_conversation(pool, &conv_key).await?;
    // Parse timestamp
    let ts: DateTime<Utc> = timestamp.parse().unwrap_or_else(|_| Utc::now());
    // Insert message
    let rec = sqlx::query!(
        r#"INSERT INTO messages (conversation_id, provider_id, direction, sent_at, received_at)
           VALUES ($1, 1, 'inbound', $2, $2) RETURNING id"#,
        convo_id,
        ts
    )
    .fetch_one(pool)
    .await?;
    // TODO: persist body + attachments in separate tables once schema extended
    Ok(rec.id)
}

async fn ensure_conversation(pool: &PgPool, key: &str) -> Result<i64> {
    // Use topic column as temporary key storage; fetch existing or create.
    if let Some(existing) = sqlx::query!(
        r#"SELECT id FROM conversations WHERE topic = $1 LIMIT 1"#,
        key
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(existing.id);
    }
    let rec = sqlx::query!(
        r#"INSERT INTO conversations (customer_id, topic) VALUES (1, $1) RETURNING id"#,
        key
    )
    .fetch_one(pool)
    .await?;
    Ok(rec.id)
}
