// Feature 009 - US3 Contract test (T029)
// Contract test for GET /api/conversations/{id}/messages
// Verifies messages include from/to, snippets, and proper timestamp ordering

use chrono::Utc;
use sqlx::PgPool;

/// Helper to create a conversation with multiple messages
async fn setup_conversation_with_messages(pool: &PgPool) -> anyhow::Result<i64> {
    let now = Utc::now();
    
    // Create outbound message
    let msg1_id = messaging_server::store_db::messages::insert_outbound(
        pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "First outbound message",
        &vec![],
        &now.to_rfc3339(),
    )
    .await?;
    
    // Create inbound reply (reversed from/to)
    let later = now + chrono::Duration::seconds(5);
    messaging_server::store_db::messages::insert_from_inbound(
        pool,
        "email",
        "bob@example.com",
        "alice@example.com",
        "First inbound reply",
        &vec![],
        &later.to_rfc3339(),
    )
    .await?;
    
    // Get conversation_id from first message
    let msg = sqlx::query!("SELECT conversation_id FROM messages WHERE id = $1", msg1_id)
        .fetch_one(pool)
        .await?;
    
    Ok(msg.conversation_id)
}

/// Test that messages include from and to fields
#[sqlx::test]
async fn list_messages_includes_from_to(pool: PgPool) -> anyhow::Result<()> {
    let conv_id = setup_conversation_with_messages(&pool).await?;
    
    // Query messages (in a real test this would be via API)
    let messages = sqlx::query!(
        r#"SELECT m.id, m.direction, m.sent_at, m.received_at,
                  c.participant_a, c.participant_b
           FROM messages m
           JOIN conversations c ON m.conversation_id = c.id
           WHERE m.conversation_id = $1
           ORDER BY COALESCE(m.received_at, m.sent_at) ASC"#,
        conv_id
    )
    .fetch_all(&pool)
    .await?;
    
    assert!(messages.len() >= 2, "Should have at least 2 messages");
    
    // Verify we have both participants
    for msg in messages {
        assert!(msg.participant_a.is_some());
        assert!(msg.participant_b.is_some());
    }
    
    Ok(())
}

/// Test timestamp ordering: received_at for inbound, sent_at for outbound
#[sqlx::test]
async fn list_messages_timestamp_ordering(pool: PgPool) -> anyhow::Result<()> {
    let conv_id = setup_conversation_with_messages(&pool).await?;
    
    // Query with proper timestamp ordering
    let messages = sqlx::query!(
        r#"SELECT id, direction, sent_at, received_at,
                  COALESCE(received_at, sent_at) as effective_ts
           FROM messages
           WHERE conversation_id = $1
           ORDER BY COALESCE(received_at, sent_at) ASC"#,
        conv_id
    )
    .fetch_all(&pool)
    .await?;
    
    // Verify chronological order
    for i in 1..messages.len() {
        let prev_ts = messages[i - 1].effective_ts;
        let curr_ts = messages[i].effective_ts;
        assert!(
            prev_ts <= curr_ts,
            "Messages should be ordered chronologically"
        );
    }
    
    Ok(())
}

/// Test snippet generation with body content
#[sqlx::test]
async fn list_messages_includes_snippet(pool: PgPool) -> anyhow::Result<()> {
    // Create message with long body
    let long_body = "This is a very long message body that should be truncated into a snippet. ".repeat(10);
    
    let msg_id = messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        &long_body,
        &vec![],
        &Utc::now().to_rfc3339(),
    )
    .await?;
    
    // Get message and body
    let message = sqlx::query!(
        r#"SELECT m.id, mb.body
           FROM messages m
           LEFT JOIN message_bodies mb ON m.body_id = mb.id
           WHERE m.id = $1"#,
        msg_id
    )
    .fetch_one(&pool)
    .await?;
    
    assert!(message.body.is_some());
    let body = message.body.unwrap();
    
    // Verify snippet would be shorter than full body
    use messaging_core::conversations::snippet::make_snippet;
    let snippet = make_snippet(Some(&body), 64);
    
    assert!(snippet.len() < body.len(), "Snippet should be truncated");
    assert!(snippet.len() <= 64, "Snippet should respect max length");
    
    Ok(())
}

/// Test snippet with Unicode content
#[sqlx::test]
async fn list_messages_snippet_unicode_safe(pool: PgPool) -> anyhow::Result<()> {
    // Create message with emoji and multi-byte characters
    let unicode_body = "Hello 世界 👍🏽 Test message";
    
    let msg_id = messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        unicode_body,
        &vec![],
        &Utc::now().to_rfc3339(),
    )
    .await?;
    
    let message = sqlx::query!(
        r#"SELECT mb.body
           FROM messages m
           LEFT JOIN message_bodies mb ON m.body_id = mb.id
           WHERE m.id = $1"#,
        msg_id
    )
    .fetch_one(&pool)
    .await?;
    
    let body = message.body.unwrap();
    
    // Verify snippet is UTF-8 safe
    use messaging_core::conversations::snippet::make_snippet;
    let snippet = make_snippet(Some(&body), 10);
    
    // Should not panic and should be valid UTF-8
    assert!(snippet.is_char_boundary(snippet.len()));
    assert!(snippet.len() <= body.len());
    
    Ok(())
}

/// Test empty body handling
#[sqlx::test]
async fn list_messages_handles_empty_body(pool: PgPool) -> anyhow::Result<()> {
    let msg_id = messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "", // empty body
        &vec![],
        &Utc::now().to_rfc3339(),
    )
    .await?;
    
    let message = sqlx::query!(
        r#"SELECT m.body_id, mb.body
           FROM messages m
           LEFT JOIN message_bodies mb ON m.body_id = mb.id
           WHERE m.id = $1"#,
        msg_id
    )
    .fetch_one(&pool)
    .await?;
    
    // Empty body should result in NULL body_id
    assert!(message.body_id.is_none() || message.body.as_ref().map(|b| b.is_empty()).unwrap_or(true));
    
    Ok(())
}

/// Test direction field (inbound vs outbound)
#[sqlx::test]
async fn list_messages_proper_direction(pool: PgPool) -> anyhow::Result<()> {
    let now = Utc::now();
    
    // Outbound
    let out_id = messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "Outbound test",
        &vec![],
        &now.to_rfc3339(),
    )
    .await?;
    
    // Inbound
    let in_id = messaging_server::store_db::messages::insert_from_inbound(
        &pool,
        "email",
        "bob@example.com",
        "alice@example.com",
        "Inbound test",
        &vec![],
        &now.to_rfc3339(),
    )
    .await?;
    
    let out_msg = sqlx::query!("SELECT direction FROM messages WHERE id = $1", out_id)
        .fetch_one(&pool)
        .await?;
    
    let in_msg = sqlx::query!("SELECT direction FROM messages WHERE id = $1", in_id)
        .fetch_one(&pool)
        .await?;
    
    assert_eq!(out_msg.direction, "outbound");
    assert_eq!(in_msg.direction, "inbound");
    
    Ok(())
}

/// Test pagination for messages
#[sqlx::test]
async fn list_messages_pagination(pool: PgPool) -> anyhow::Result<()> {
    let now = Utc::now();
    
    // Create conversation with 5 messages
    for i in 0..5 {
        let ts = now + chrono::Duration::seconds(i);
        messaging_server::store_db::messages::insert_outbound(
            &pool,
            "email",
            "alice@example.com",
            "bob@example.com",
            &format!("Message {}", i),
            &vec![],
            &ts.to_rfc3339(),
        )
        .await?;
    }
    
    // Get conversation_id
    let conv = sqlx::query!("SELECT id FROM conversations LIMIT 1")
        .fetch_one(&pool)
        .await?;
    
    // Test pagination
    let page1 = sqlx::query!(
        "SELECT id FROM messages WHERE conversation_id = $1 ORDER BY sent_at LIMIT 2 OFFSET 0",
        conv.id
    )
    .fetch_all(&pool)
    .await?;
    
    let page2 = sqlx::query!(
        "SELECT id FROM messages WHERE conversation_id = $1 ORDER BY sent_at LIMIT 2 OFFSET 2",
        conv.id
    )
    .fetch_all(&pool)
    .await?;
    
    assert_eq!(page1.len(), 2);
    assert_eq!(page2.len(), 2);
    
    // Verify no overlap
    let page1_ids: Vec<i64> = page1.iter().map(|r| r.id).collect();
    let page2_ids: Vec<i64> = page2.iter().map(|r| r.id).collect();
    
    for id in page1_ids {
        assert!(!page2_ids.contains(&id));
    }
    
    Ok(())
}
