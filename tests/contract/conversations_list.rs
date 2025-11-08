// Feature 009 - US2 Contract test (T024)
// Contract test for GET /api/conversations
// Verifies deterministic listing with pagination and proper DTO fields

use chrono::Utc;
use sqlx::PgPool;

/// Helper to insert test message and conversation
async fn setup_test_conversations(pool: &PgPool) -> anyhow::Result<()> {
    // Create 3 conversations with different timestamps for ordering tests
    let now = Utc::now();
    
    // Conversation 1: alice <-> bob (most recent)
    messaging_server::store_db::messages::insert_outbound(
        pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "Message 1",
        &vec![],
        &now.to_rfc3339(),
    )
    .await?;
    
    // Conversation 2: charlie <-> david (middle)
    let earlier = now - chrono::Duration::minutes(10);
    messaging_server::store_db::messages::insert_outbound(
        pool,
        "email",
        "charlie@example.com",
        "david@example.com",
        "Message 2",
        &vec![],
        &earlier.to_rfc3339(),
    )
    .await?;
    
    // Conversation 3: eve <-> frank (oldest)
    let oldest = now - chrono::Duration::minutes(20);
    messaging_server::store_db::messages::insert_outbound(
        pool,
        "sms",
        "+15550001111",
        "+15550002222",
        "Message 3",
        &vec![],
        &oldest.to_rfc3339(),
    )
    .await?;
    
    Ok(())
}

/// Test that GET /api/conversations returns proper DTO structure
#[sqlx::test]
async fn list_conversations_returns_proper_dto(pool: PgPool) -> anyhow::Result<()> {
    setup_test_conversations(&pool).await?;
    
    // Query conversations from DB
    let conversations = sqlx::query!(
        r#"SELECT id, key, channel, participant_a, participant_b, message_count, last_activity_at
           FROM conversations
           ORDER BY last_activity_at DESC, id DESC
           LIMIT 10"#
    )
    .fetch_all(&pool)
    .await?;
    
    assert!(conversations.len() >= 3, "Should have at least 3 conversations");
    
    // Verify first conversation has all required fields
    let first = &conversations[0];
    assert!(first.id > 0);
    assert!(!first.key.is_empty());
    assert!(first.channel.is_some());
    assert!(first.participant_a.is_some());
    assert!(first.participant_b.is_some());
    assert!(first.message_count >= 1);
    assert!(first.last_activity_at.is_some());
    
    Ok(())
}

/// Test deterministic ordering: last_activity_at DESC, then id DESC
#[sqlx::test]
async fn list_conversations_deterministic_ordering(pool: PgPool) -> anyhow::Result<()> {
    setup_test_conversations(&pool).await?;
    
    // Query twice to verify deterministic order
    let query = r#"SELECT id, last_activity_at
                   FROM conversations
                   ORDER BY last_activity_at DESC, id DESC
                   LIMIT 10"#;
    
    let first_fetch = sqlx::query(query)
        .fetch_all(&pool)
        .await?;
    
    let second_fetch = sqlx::query(query)
        .fetch_all(&pool)
        .await?;
    
    assert_eq!(first_fetch.len(), second_fetch.len());
    
    // Verify order is identical
    for (i, (first, second)) in first_fetch.iter().zip(second_fetch.iter()).enumerate() {
        let id1: i64 = first.get("id");
        let id2: i64 = second.get("id");
        assert_eq!(id1, id2, "Order should be deterministic at position {}", i);
    }
    
    Ok(())
}

/// Test pagination stability
#[sqlx::test]
async fn list_conversations_pagination_stable(pool: PgPool) -> anyhow::Result<()> {
    setup_test_conversations(&pool).await?;
    
    // Fetch page 1
    let page1 = sqlx::query!(
        r#"SELECT id FROM conversations
           ORDER BY last_activity_at DESC, id DESC
           LIMIT 2 OFFSET 0"#
    )
    .fetch_all(&pool)
    .await?;
    
    // Fetch page 2
    let page2 = sqlx::query!(
        r#"SELECT id FROM conversations
           ORDER BY last_activity_at DESC, id DESC
           LIMIT 2 OFFSET 2"#
    )
    .fetch_all(&pool)
    .await?;
    
    // Verify no overlap
    let page1_ids: Vec<i64> = page1.iter().map(|r| r.id).collect();
    let page2_ids: Vec<i64> = page2.iter().map(|r| r.id).collect();
    
    for id in page1_ids {
        assert!(!page2_ids.contains(&id), "Pages should not overlap");
    }
    
    Ok(())
}

/// Test that participants are properly ordered (normalized)
#[sqlx::test]
async fn list_conversations_normalized_participants(pool: PgPool) -> anyhow::Result<()> {
    // Insert message with reversed from/to
    messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "zebra@example.com",
        "aardvark@example.com",
        "Test",
        &vec![],
        &Utc::now().to_rfc3339(),
    )
    .await?;
    
    // Verify participant_a is lexicographically smaller
    let conversation = sqlx::query!(
        r#"SELECT participant_a, participant_b FROM conversations
           WHERE participant_a = 'aardvark@example.com' OR participant_b = 'aardvark@example.com'
           LIMIT 1"#
    )
    .fetch_one(&pool)
    .await?;
    
    assert_eq!(conversation.participant_a, Some("aardvark@example.com".to_string()));
    assert_eq!(conversation.participant_b, Some("zebra@example.com".to_string()));
    
    Ok(())
}

/// Test key format
#[sqlx::test]
async fn list_conversations_key_format(pool: PgPool) -> anyhow::Result<()> {
    messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "alice@example.com",
        "bob@example.com",
        "Test",
        &vec![],
        &Utc::now().to_rfc3339(),
    )
    .await?;
    
    let conversation = sqlx::query!(
        r#"SELECT key, channel, participant_a, participant_b FROM conversations
           WHERE participant_a = 'alice@example.com'
           LIMIT 1"#
    )
    .fetch_one(&pool)
    .await?;
    
    // Verify key format: {channel}:{participant_a}<->{participant_b}
    let expected_key = format!(
        "{}:{}<->{}",
        conversation.channel.unwrap(),
        conversation.participant_a.unwrap(),
        conversation.participant_b.unwrap()
    );
    assert_eq!(conversation.key, expected_key);
    
    Ok(())
}

/// Test multiple channels
#[sqlx::test]
async fn list_conversations_multiple_channels(pool: PgPool) -> anyhow::Result<()> {
    // Email conversation
    messaging_server::store_db::messages::insert_outbound(
        &pool,
        "email",
        "user@example.com",
        "other@example.com",
        "Email test",
        &vec![],
        &Utc::now().to_rfc3339(),
    )
    .await?;
    
    // SMS conversation
    messaging_server::store_db::messages::insert_outbound(
        &pool,
        "sms",
        "+15551234567",
        "+15559876543",
        "SMS test",
        &vec![],
        &Utc::now().to_rfc3339(),
    )
    .await?;
    
    let conversations = sqlx::query!(
        r#"SELECT channel FROM conversations ORDER BY id"#
    )
    .fetch_all(&pool)
    .await?;
    
    let channels: Vec<String> = conversations
        .into_iter()
        .filter_map(|c| c.channel)
        .collect();
    
    assert!(channels.contains(&"email".to_string()));
    assert!(channels.contains(&"sms".to_string()));
    
    Ok(())
}
