use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, warn};

/// Skeleton backfill logic for populating messages.conversation_ref from conversations.id
/// ensuring alignment with new durable key-based conversations schema.
/// This is a placeholder; future logic may derive key components if missing.
pub async fn backfill_conversation_refs(pool: &PgPool, batch_size: i64) -> Result<()> {
    // Basic loop updating NULL refs in batches to avoid long-running locks.
    loop {
        let updated = sqlx::query(
            r#"UPDATE messages SET conversation_ref = conversation_id
               WHERE id IN (
                   SELECT id FROM messages WHERE conversation_ref IS NULL LIMIT $1
               )"#,
        )
        .bind(batch_size)
        .execute(pool)
        .await?;
        let count = updated.rows_affected();
        if count == 0 { break; }
        info!(target="backfill", affected = count, "backfilled batch of conversation_ref values");
        if count < batch_size as u64 { break; }
    }
    // Verify remaining NULL count (non-fatal)
    if let Ok(Some(row)) = sqlx::query("SELECT COUNT(*)::BIGINT AS cnt FROM messages WHERE conversation_ref IS NULL")
        .fetch_optional(pool).await {
        let remaining: i64 = row.get("cnt");
        if remaining > 0 { warn!(target="backfill", remaining, "remaining NULL conversation_ref rows after backfill skeleton"); }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn skeleton_runs(pool: PgPool) {
        // Should run without error even on empty messages table.
        assert!(backfill_conversation_refs(&pool, 100).await.is_ok());
    }
}
