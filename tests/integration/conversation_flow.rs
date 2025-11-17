// Feature 009 - US1 Integration test (T015)
// Deterministic fixture: outbound + inbound pair results in one conversation
// Verifies that messages sent between the same participants on the same channel
// are linked to a single conversation with accurate message_count and last_activity

use chrono::Utc;
use sqlx::PgPool;

/// Test helper to insert a test message and return the conversation_id
async fn insert_test_message(
    pool: &PgPool,
    channel: &str,
    from: &str,
    to: &str,
    body: &str,
    direction: &str,
) -> anyhow::Result<i64> {
    let timestamp = Utc::now().to_rfc3339();
    let attachments: Vec<String> = vec![];
    
    match direction {
        "inbound" => {
            messaging_server::store_db::messages::insert_from_inbound(
                pool,
                channel,
                from,
                to,
                body,
                &attachments,
                &timestamp,
            )
            .await
        }
        "outbound" => {
            messaging_server::store_db::messages::insert_outbound(
                pool,
                channel,
                from,
                to,
                body,
                &attachments,
                &timestamp,
            )
            .await
        }
        _ => Err(anyhow::anyhow!("invalid direction")),
    }
}

/// Test that outbound + inbound pair for same participants/channel results in single conversation
#[sqlx::test]
async fn outbound_inbound_pair_creates_single_conversation(pool: PgPool) -> anyhow::Result<()> {
    // Send outbound message
    let msg1_id = insert_test_message(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "Hello Bob!",
        "outbound",
    )
    .await?;

    // Send inbound reply (reversed from/to)
    let msg2_id = insert_test_message(
        &pool,
        "email",
        "bob@example.com",
        "alice@example.com",
        "Hi Alice!",
        "inbound",
    )
    .await?;

    // Verify messages exist
    assert!(msg1_id > 0);
    assert!(msg2_id > 0);

    // Query conversations - should be exactly 1
    let conversations = sqlx::query!(
        r#"SELECT id, channel, participant_a, participant_b, message_count, key
           FROM conversations
           WHERE (participant_a = 'alice@example.com' AND participant_b = 'bob@example.com')
              OR (participant_a = 'bob@example.com' AND participant_b = 'alice@example.com')"#
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(
        conversations.len(),
        1,
        "Expected exactly 1 conversation for alice/bob pair"
    );

    let convo = &conversations[0];
    assert_eq!(convo.channel, "email");
    assert_eq!(convo.message_count, 2, "Expected message_count = 2");
    
    // Verify normalized ordering (lexicographically smaller first)
    assert_eq!(convo.participant_a, "alice@example.com");
    assert_eq!(convo.participant_b, "bob@example.com");
    assert_eq!(convo.key, "email:alice@example.com<->bob@example.com");

    // Verify both messages reference the same conversation_id
    let msg1_convo = sqlx::query!(
        "SELECT conversation_id FROM messages WHERE id = $1",
        msg1_id
    )
    .fetch_one(&pool)
    .await?;
    
    let msg2_convo = sqlx::query!(
        "SELECT conversation_id FROM messages WHERE id = $1",
        msg2_id
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(
        msg1_convo.conversation_id,
        msg2_convo.conversation_id,
        "Both messages should reference the same conversation"
    );
    assert_eq!(msg1_convo.conversation_id, convo.id);

    Ok(())
}

/// Test normalization: plus-addressing for email should be normalized
#[sqlx::test]
async fn email_plus_addressing_normalization(pool: PgPool) -> anyhow::Result<()> {
    // Send to user+tag@example.com
    let msg1_id = insert_test_message(
        &pool,
        "email",
        "alice@example.com",
        "user+tag@example.com",
        "Message 1",
        "outbound",
    )
    .await?;

    // Send to user@example.com (base address)
    let msg2_id = insert_test_message(
        &pool,
        "email",
        "alice@example.com",
        "user@example.com",
        "Message 2",
        "outbound",
    )
    .await?;

    // Verify only 1 conversation exists (plus-tag normalized)
    let conversations = sqlx::query!(
        r#"SELECT id, message_count FROM conversations
           WHERE participant_a = 'alice@example.com' AND participant_b = 'user@example.com'"#
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(
        conversations.len(),
        1,
        "Plus-addressing should normalize to same conversation"
    );
    assert_eq!(conversations[0].message_count, 2);

    Ok(())
}

/// Test phone normalization: formatting should be stripped
#[sqlx::test]
async fn phone_formatting_normalization(pool: PgPool) -> anyhow::Result<()> {
    // Different formatting, same number
    let msg1_id = insert_test_message(
        &pool,
        "sms",
        "+1 (555) 000-1234",
        "+1 (555) 000-5678",
        "Message 1",
        "outbound",
    )
    .await?;

    let msg2_id = insert_test_message(
        &pool,
        "sms",
        "+15550001234",
        "+15550005678",
        "Message 2",
        "outbound",
    )
    .await?;

    // Verify only 1 conversation exists
    let conversations = sqlx::query!(
        r#"SELECT id, message_count FROM conversations
           WHERE channel = 'sms'"#
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(
        conversations.len(),
        1,
        "Phone formatting should normalize to same conversation"
    );
    assert_eq!(conversations[0].message_count, 2);

    Ok(())
}

/// Test idempotency: duplicate message should not increment count
#[sqlx::test]
async fn duplicate_message_idempotency(pool: PgPool) -> anyhow::Result<()> {
    let timestamp = Utc::now().to_rfc3339();
    let attachments: Vec<String> = vec![];

    // Insert first message
    let msg1_id = messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "Test message",
        &attachments,
        &timestamp,
    )
    .await?;

    // Insert duplicate (same timestamp, body, participants)
    let msg2_id = messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "Test message",
        &attachments,
        &timestamp,
    )
    .await?;

    // Should return same message ID
    assert_eq!(msg1_id, msg2_id, "Duplicate should return existing message ID");

    // Verify conversation has message_count = 1
    let conversations = sqlx::query!(
        r#"SELECT message_count FROM conversations
           WHERE participant_a = 'alice@example.com' AND participant_b = 'bob@example.com'"#
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(conversations.len(), 1);
    assert_eq!(
        conversations[0].message_count, 1,
        "Duplicate message should not increment count"
    );

    Ok(())
}
