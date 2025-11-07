// Placeholder DB conversation/message listing until full mapping implemented.
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub struct ConversationSummary {
    pub id: i64,
    pub topic: Option<String>,
    pub message_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
}

// Legacy MessageRow removed; using ConversationMessage below.

pub async fn list_conversations(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ConversationSummary>> {
    let rows = sqlx::query!(
        r#"SELECT c.id, c.topic, COUNT(m.id)::bigint AS message_count, MAX(COALESCE(m.received_at, m.sent_at)) AS last_message_at
            FROM conversations c
            LEFT JOIN messages m ON m.conversation_id = c.id
            GROUP BY c.id
            ORDER BY last_message_at DESC NULLS LAST, c.id ASC
            LIMIT $1 OFFSET $2"#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationSummary {
            id: r.id,
            topic: r.topic,
            message_count: r.message_count.unwrap_or(0),
            last_message_at: r.last_message_at,
        })
        .collect())
}

#[derive(Debug)]
pub struct ConversationMessage {
    pub id: i64,
    pub direction: String,
    pub provider_id: i64,
    pub sent_at: DateTime<Utc>,
    pub received_at: Option<DateTime<Utc>>,
}

pub async fn list_messages(
    pool: &PgPool,
    conversation_id: i64,
    limit: i64,
    offset: i64,
) -> Result<Vec<ConversationMessage>> {
    let rows = sqlx::query!(
        r#"SELECT m.id, m.direction, m.provider_id, m.sent_at, m.received_at
            FROM messages m
            WHERE m.conversation_id = $1
            ORDER BY COALESCE(m.received_at, m.sent_at) DESC
            LIMIT $2 OFFSET $3"#,
        conversation_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationMessage {
            id: r.id,
            direction: r.direction,
            provider_id: r.provider_id,
            sent_at: r.sent_at,
            received_at: r.received_at,
        })
        .collect())
}

pub async fn conversations_total(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query!(r#"SELECT COUNT(*) as count FROM conversations"#)
        .fetch_one(pool)
        .await?;
    Ok(row.count.unwrap_or(0))
}

pub async fn messages_total(pool: &PgPool, conversation_id: i64) -> Result<i64> {
    let row = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM messages WHERE conversation_id = $1"#,
        conversation_id
    )
    .fetch_one(pool)
    .await?;
    Ok(row.count.unwrap_or(0))
}
