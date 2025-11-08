// DB conversation/message listing with durable schema fallback.
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

#[derive(Debug)]
pub struct ConversationSummary {
    pub id: i64,
    pub key: String,
    pub message_count: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
}

// Legacy MessageRow removed; using ConversationMessage below.

pub async fn list_conversations(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ConversationSummary>> {
    // Try durable schema first
    match sqlx::query(
        r#"SELECT id, key, message_count, last_activity_at
           FROM conversations
           ORDER BY last_activity_at DESC NULLS LAST, id ASC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => Ok(rows
            .into_iter()
            .map(|r| ConversationSummary {
                id: r.get("id"),
                key: r.get::<String, _>("key"),
                message_count: r.get::<i64, _>("message_count"),
                last_activity_at: r.get::<Option<DateTime<Utc>>, _>("last_activity_at"),
            })
            .collect()),
        Err(_) => {
            // Fallback to legacy schema (topic + computed counts)
            let rows = sqlx::query(
                r#"SELECT c.id, c.topic, COUNT(m.id)::bigint AS message_count,
                          MAX(COALESCE(m.received_at, m.sent_at)) AS last_message_at
                   FROM conversations c
                   LEFT JOIN messages m ON m.conversation_id = c.id
                   GROUP BY c.id, c.topic
                   ORDER BY last_message_at DESC NULLS LAST, c.id ASC
                   LIMIT $1 OFFSET $2"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            Ok(rows
                .into_iter()
                .map(|r| ConversationSummary {
                    id: r.get("id"),
                    key: r.get::<Option<String>, _>("topic").unwrap_or_default(),
                    message_count: r.get::<Option<i64>, _>("message_count").unwrap_or(0),
                    last_activity_at: r.get::<Option<DateTime<Utc>>, _>("last_message_at"),
                })
                .collect())
        }
    }
}

#[derive(Debug)]
pub struct ConversationMessage {
    pub id: i64,
    pub direction: String,
    pub provider_id: i64,
    pub sent_at: DateTime<Utc>,
    pub received_at: Option<DateTime<Utc>>,
    pub body: Option<String>,
}

pub async fn list_messages(
    pool: &PgPool,
    conversation_id: i64,
    limit: i64,
    offset: i64,
) -> Result<Vec<ConversationMessage>> {
    let rows = sqlx::query(
        r#"SELECT m.id, m.direction, m.provider_id, m.sent_at, m.received_at, b.body
            FROM messages m
            LEFT JOIN message_bodies b ON m.body_id = b.id
            WHERE m.conversation_id = $1
            ORDER BY COALESCE(m.received_at, m.sent_at) DESC
            LIMIT $2 OFFSET $3"#,
    )
    .bind(conversation_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationMessage {
            id: r.get("id"),
            direction: r.get("direction"),
            provider_id: r.get("provider_id"),
            sent_at: r.get("sent_at"),
            received_at: r.get("received_at"),
            body: r.get("body"),
        })
        .collect())
}

pub async fn conversations_total(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query(r#"SELECT COUNT(*) as count FROM conversations"#)
        .fetch_one(pool)
        .await?;
    let count: i64 = row.get::<Option<i64>, _>("count").unwrap_or(0);
    Ok(count)
}

pub async fn messages_total(pool: &PgPool, conversation_id: i64) -> Result<i64> {
    let row = sqlx::query(r#"SELECT COUNT(*) as count FROM messages WHERE conversation_id = $1"#)
        .bind(conversation_id)
        .fetch_one(pool)
        .await?;
    let count: i64 = row.get::<Option<i64>, _>("count").unwrap_or(0);
    Ok(count)
}
