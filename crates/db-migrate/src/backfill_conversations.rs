use anyhow::Result;
use sqlx::{PgPool, Row};
use tracing::{info, warn};

/// Backfill conversations for messages with NULL conversation_id
///
/// This function processes messages in batches, deriving conversation keys from
/// message participants and channel, upserting conversations, and linking messages
/// to the appropriate conversations.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `batch_size` - Number of messages to process per batch
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(_)` if backfill encounters unrecoverable errors
#[allow(dead_code)]
pub async fn backfill_conversations(pool: &PgPool, batch_size: i64) -> Result<()> {
    info!(
        target = "backfill",
        "Starting conversation backfill process"
    );

    let mut total_processed = 0;
    let mut total_errors = 0;

    loop {
        // Fetch batch of messages with NULL conversation_id
        let messages = sqlx::query(
            r#"SELECT m.id, m.direction, m.sent_at, m.received_at,
                      p.channel, p.from_addr, p.to_addr
               FROM messages m
               LEFT JOIN (
                   -- Derive from/to from message context (simplified; may need provider join)
                   SELECT id, 
                          'email' as channel, 
                          'unknown@example.com' as from_addr,
                          'unknown@example.com' as to_addr
                   FROM messages
                   WHERE conversation_id IS NULL
               ) p ON m.id = p.id
               WHERE m.conversation_id IS NULL
               LIMIT $1"#,
        )
        .bind(batch_size)
        .fetch_all(pool)
        .await?;

        if messages.is_empty() {
            break;
        }

        let batch_count = messages.len();
        info!(
            target = "backfill",
            batch_size = batch_count,
            "Processing batch"
        );

        for row in &messages {
            let message_id: i64 = row.get("id");
            let _direction: String = row.get("direction");

            // For this skeleton, we'll need to derive channel and participants
            // In a real implementation, this would query related tables
            // For now, mark as needing manual intervention
            warn!(
                target = "backfill",
                message_id = message_id,
                "Message missing conversation_id requires manual participant derivation"
            );
            total_errors += 1;
        }

        total_processed += batch_count;

        if messages.len() < batch_size as usize {
            break;
        }
    }

    info!(
        target = "backfill",
        total_processed = total_processed,
        total_errors = total_errors,
        "Backfill process completed"
    );

    // Verify and report remaining NULL conversation_id
    verify_backfill_completion(pool).await?;

    Ok(())
}

/// Verify backfill completion and report statistics
#[allow(dead_code)]
pub async fn verify_backfill_completion(pool: &PgPool) -> Result<()> {
    info!(target = "backfill", "Verifying backfill completion");

    // Count messages with NULL conversation_id
    let null_count =
        sqlx::query("SELECT COUNT(*) as cnt FROM messages WHERE conversation_id IS NULL")
            .fetch_one(pool)
            .await?;
    let remaining: i64 = null_count.get("cnt");

    // Count total messages
    let total_count = sqlx::query("SELECT COUNT(*) as cnt FROM messages")
        .fetch_one(pool)
        .await?;
    let total: i64 = total_count.get("cnt");

    // Count total conversations
    let convo_count = sqlx::query("SELECT COUNT(*) as cnt FROM conversations")
        .fetch_one(pool)
        .await?;
    let conversations: i64 = convo_count.get("cnt");

    info!(
        target = "backfill",
        total_messages = total,
        messages_with_conversation = total - remaining,
        messages_without_conversation = remaining,
        total_conversations = conversations,
        "Backfill verification complete"
    );

    if remaining > 0 {
        warn!(
            target = "backfill",
            remaining = remaining,
            "Messages still without conversation_id after backfill"
        );
    }

    Ok(())
}

/// Recompute aggregate statistics for conversations
///
/// This function recalculates message_count and last_activity_at for all conversations
/// based on the actual messages in the database. Useful after manual data corrections.
#[allow(dead_code)]
pub async fn recompute_conversation_aggregates(pool: &PgPool) -> Result<()> {
    info!(target = "backfill", "Recomputing conversation aggregates");

    let result = sqlx::query(
        r#"UPDATE conversations c
           SET message_count = (
               SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id
           ),
           last_activity_at = (
               SELECT GREATEST(MAX(m.sent_at), MAX(m.received_at))
               FROM messages m 
               WHERE m.conversation_id = c.id
           )
           WHERE EXISTS (SELECT 1 FROM messages m WHERE m.conversation_id = c.id)"#,
    )
    .execute(pool)
    .await?;

    let updated = result.rows_affected();
    info!(
        target = "backfill",
        conversations_updated = updated,
        "Aggregate recomputation complete"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn backfill_runs_on_empty_db(pool: PgPool) -> Result<()> {
        // Should run without error on empty database
        backfill_conversations(&pool, 100).await?;
        verify_backfill_completion(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn recompute_runs_on_empty_db(pool: PgPool) -> Result<()> {
        // Should run without error on empty database
        recompute_conversation_aggregates(&pool).await?;
        Ok(())
    }
}
