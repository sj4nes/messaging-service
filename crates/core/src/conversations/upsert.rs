use crate::conversations::{key::derive_key, key::ChannelKind, ConversationKey};
use sqlx::{PgPool, Postgres, Row, Transaction};
use tracing::{error, info, instrument};

#[derive(Debug)]
pub enum UpsertOutcome {
    Created(i64, ConversationKey),
    Reused(i64, ConversationKey),
    Failed(String),
}

/// Upsert conversation returning id and derived key. Message count & last_activity updated by caller.
#[instrument(skip(pool))]
pub async fn upsert_conversation(
    pool: &PgPool,
    channel: ChannelKind,
    from: &str,
    to: &str,
    activity_ts: sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
) -> UpsertOutcome {
    let k = derive_key(channel.clone(), from, to);
    // Use transaction for atomicity
    let mut tx: Transaction<'_, Postgres> = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return UpsertOutcome::Failed(format!("tx begin error: {e}")),
    };
    // Attempt select first
    let rec = sqlx::query(
        r#"SELECT id, message_count, last_activity_at FROM conversations
            WHERE channel = $1 AND participant_a = $2 AND participant_b = $3"#,
    )
    .bind(&k.channel)
    .bind(&k.participant_a)
    .bind(&k.participant_b)
    .fetch_optional(&mut *tx)
    .await;
    match rec {
        Ok(Some(row)) => {
            // Update last activity only (message_count handled by caller after message insert)
            let existing_id: i64 = row.get("id");
            if let Err(e) = sqlx::query(
                "UPDATE conversations SET last_activity_at = GREATEST(last_activity_at, $1) WHERE id = $2",
            )
            .bind(activity_ts)
            .bind(existing_id)
            .execute(&mut *tx)
            .await {
                error!(conversation_id = existing_id, err = %e, "failed to update last_activity_at");
                let _ = tx.rollback().await;
                return UpsertOutcome::Failed(format!("update last_activity error: {e}"));
            }
            if let Err(e) = tx.commit().await {
                return UpsertOutcome::Failed(format!("commit error: {e}"));
            }
            UpsertOutcome::Reused(existing_id, k)
        }
        Ok(None) => {
            // Insert new conversation
            let ins = sqlx::query(
                r#"INSERT INTO conversations(channel, participant_a, participant_b, message_count, last_activity_at, key)
                   VALUES ($1,$2,$3,0,$4,$5)
                   ON CONFLICT (channel, participant_a, participant_b) DO UPDATE
                     SET last_activity_at = GREATEST(conversations.last_activity_at, EXCLUDED.last_activity_at)
                   RETURNING id"#,
            )
            .bind(&k.channel)
            .bind(&k.participant_a)
            .bind(&k.participant_b)
            .bind(activity_ts)
            .bind(&k.key)
            .fetch_one(&mut *tx)
            .await;
            match ins {
                Ok(row) => {
                    let new_id: i64 = row.get("id");
                    if let Err(e) = tx.commit().await {
                        return UpsertOutcome::Failed(format!("commit error: {e}"));
                    }
                    info!(conversation_id = new_id, key = %k.key, "conversation created or reused via upsert");
                    UpsertOutcome::Created(new_id, k)
                }
                Err(e) => {
                    error!(key = %k.key, err = %e, "conversation upsert error");
                    let _ = tx.rollback().await;
                    UpsertOutcome::Failed(format!("upsert error: {e}"))
                }
            }
        }
        Err(e) => {
            let _ = tx.rollback().await;
            UpsertOutcome::Failed(format!("select error: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // NOTE: These tests require a test database. They should be integrated with existing test harness.
    // Placeholder demonstrating structure.
    #[sqlx::test]
    async fn upsert_creates_then_reuses(pool: PgPool) {
        let ts = sqlx::types::chrono::Utc::now();
        let first = upsert_conversation(
            &pool,
            ChannelKind::Email,
            "a@example.com",
            "b@example.com",
            ts,
        )
        .await;
        match first {
            UpsertOutcome::Created(id, _) => assert!(id > 0),
            _ => panic!("expected create"),
        }
        let again = upsert_conversation(
            &pool,
            ChannelKind::Email,
            "b@example.com",
            "a@example.com",
            ts,
        )
        .await; // reversed order
        match again {
            UpsertOutcome::Reused(id, _) => assert!(id > 0),
            _ => panic!("expected reuse"),
        }
    }
}
