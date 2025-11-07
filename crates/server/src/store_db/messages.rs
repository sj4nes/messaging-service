use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::{instrument, warn};

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
    // Insert body row if present
    let body_id: Option<i64> = if body.is_empty() {
        None
    } else {
        let b = sqlx::query(r#"INSERT INTO message_bodies (body) VALUES ($1) RETURNING id"#)
            .bind(body)
            .fetch_one(pool)
            .await?;
        Some(b.get::<i64, _>("id"))
    };
    // Insert message referencing body
    let rec = sqlx::query!(
        r#"INSERT INTO messages (conversation_id, provider_id, direction, sent_at, received_at, body_id)
           VALUES ($1, 1, 'inbound', $2, $2, $3) RETURNING id"#,
        convo_id,
        ts,
        body_id
    )
    .fetch_one(pool)
    .await?;
    let message_id = rec.id;
    // Persist attachments (URLs) if any
    for url in attachments {
        match sqlx::query(r#"INSERT INTO attachment_urls (url) VALUES ($1) RETURNING id"#)
            .bind(url)
            .fetch_one(pool)
            .await
        {
            Ok(a) => {
                let attachment_id: i64 = a.get("id");
                if let Err(e) = sqlx::query(
                    r#"INSERT INTO message_attachment_urls (message_id, attachment_url_id) VALUES ($1, $2)"#,
                )
                .bind(message_id)
                .bind(attachment_id)
                .execute(pool)
                .await
                {
                    warn!(target="server", event="attach_link_fail", error=?e, message_id, url=%url, "failed to link attachment; continuing");
                }
            }
            Err(e) => {
                // Likely migration 0008 not yet applied; log and continue without attachments
                warn!(target="server", event="attach_insert_fail", error=?e, message_id, url=%url, "failed to persist attachment; continuing");
                continue;
            }
        }
    }
    Ok(message_id)
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
