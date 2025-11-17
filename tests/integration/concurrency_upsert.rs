// Feature 009 - US1 Integration test (T016)
// Concurrency test: 100 inserts same key → 1 conversation, count=100
// Verifies that concurrent message inserts for the same conversation are handled correctly

use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Test that 100 concurrent inserts for same participants/channel result in single conversation
#[sqlx::test]
async fn concurrent_inserts_single_conversation(pool: PgPool) -> anyhow::Result<()> {
    let pool = Arc::new(pool);
    let mut join_set = JoinSet::new();

    // Spawn 100 concurrent tasks inserting messages between same participants
    for i in 0..100 {
        let pool_clone = Arc::clone(&pool);
        join_set.spawn(async move {
            let timestamp = Utc::now().to_rfc3339();
            let attachments: Vec<String> = vec![];
            let body = format!("Concurrent message {}", i);
            
            messaging_server::store_db::messages::insert_outbound(
                &pool_clone,
                "email",
                "concurrent_a@example.com",
                "concurrent_b@example.com",
                &body,
                &attachments,
                &timestamp,
            )
            .await
        });
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    let mut message_ids = Vec::new();
    
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(msg_id)) => {
                success_count += 1;
                message_ids.push(msg_id);
            }
            Ok(Err(e)) => {
                eprintln!("Message insert failed: {}", e);
            }
            Err(e) => {
                eprintln!("Task panicked: {}", e);
            }
        }
    }

    assert_eq!(success_count, 100, "All 100 inserts should succeed");

    // Verify exactly 1 conversation exists
    let conversations = sqlx::query!(
        r#"SELECT id, message_count FROM conversations
           WHERE participant_a = 'concurrent_a@example.com' 
             AND participant_b = 'concurrent_b@example.com'"#
    )
    .fetch_all(pool.as_ref())
    .await?;

    assert_eq!(
        conversations.len(),
        1,
        "Expected exactly 1 conversation despite concurrent inserts"
    );

    let convo = &conversations[0];
    assert_eq!(
        convo.message_count, 100,
        "Expected message_count = 100 from concurrent inserts"
    );

    // Verify all messages reference the same conversation_id
    for msg_id in message_ids {
        let msg = sqlx::query!(
            "SELECT conversation_id FROM messages WHERE id = $1",
            msg_id
        )
        .fetch_one(pool.as_ref())
        .await?;
        
        assert_eq!(
            msg.conversation_id, convo.id,
            "All messages should reference the same conversation"
        );
    }

    Ok(())
}

/// Test concurrent inserts with different participants create separate conversations
#[sqlx::test]
async fn concurrent_inserts_different_conversations(pool: PgPool) -> anyhow::Result<()> {
    let pool = Arc::new(pool);
    let mut join_set = JoinSet::new();

    // Spawn 50 tasks for 5 different conversation pairs (10 messages each)
    for pair_id in 0..5 {
        for i in 0..10 {
            let pool_clone = Arc::clone(&pool);
            join_set.spawn(async move {
                let timestamp = Utc::now().to_rfc3339();
                let attachments: Vec<String> = vec![];
                let from = format!("user_a_{}@example.com", pair_id);
                let to = format!("user_b_{}@example.com", pair_id);
                let body = format!("Message {} for pair {}", i, pair_id);
                
                messaging_server::store_db::messages::insert_outbound(
                    &pool_clone,
                    "email",
                    &from,
                    &to,
                    &body,
                    &attachments,
                    &timestamp,
                )
                .await
            });
        }
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(_)) = result {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 50, "All 50 inserts should succeed");

    // Verify exactly 5 conversations exist
    let conversations = sqlx::query!(
        r#"SELECT id, participant_a, participant_b, message_count 
           FROM conversations
           WHERE participant_a LIKE 'user_a_%@example.com'
           ORDER BY participant_a"#
    )
    .fetch_all(pool.as_ref())
    .await?;

    assert_eq!(
        conversations.len(),
        5,
        "Expected exactly 5 conversations for 5 different pairs"
    );

    // Verify each conversation has 10 messages
    for convo in conversations {
        assert_eq!(
            convo.message_count, 10,
            "Each conversation should have 10 messages"
        );
    }

    Ok(())
}

/// Test race condition: concurrent upsert for same conversation key
#[sqlx::test]
async fn concurrent_upsert_race_condition(pool: PgPool) -> anyhow::Result<()> {
    let pool = Arc::new(pool);
    let mut join_set = JoinSet::new();

    // Spawn 20 concurrent tasks that will race to create the same conversation
    // All with slightly different timestamps to test last_activity_at updates
    for i in 0..20 {
        let pool_clone = Arc::clone(&pool);
        join_set.spawn(async move {
            // Add small delay variation to create race conditions
            if i % 2 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_micros(i * 10)).await;
            }
            
            let timestamp = Utc::now().to_rfc3339();
            let attachments: Vec<String> = vec![];
            let body = format!("Race message {}", i);
            
            messaging_server::store_db::messages::insert_outbound(
                &pool_clone,
                "sms",
                "+1555RACE001",
                "+1555RACE002",
                &body,
                &attachments,
                &timestamp,
            )
            .await
        });
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(_)) = result {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 20, "All 20 inserts should succeed despite race");

    // Verify exactly 1 conversation exists
    let conversations = sqlx::query!(
        r#"SELECT id, message_count, channel FROM conversations
           WHERE participant_a = '+1555RACE001' AND participant_b = '+1555RACE002'"#
    )
    .fetch_all(pool.as_ref())
    .await?;

    assert_eq!(
        conversations.len(),
        1,
        "Race condition should still result in exactly 1 conversation"
    );

    assert_eq!(conversations[0].message_count, 20);
    assert_eq!(conversations[0].channel, "sms");

    Ok(())
}
