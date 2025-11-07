use sqlx::{PgPool, Row};
use tracing::info;

/// Idempotent bootstrap seeding for local dev & tests when `DATABASE_URL` is set.
/// Ensures a default customer (id=1), provider (id=1), a conversation, one inbound message with body & attachment.
/// Safe to call on every startup; uses existence checks.
pub async fn seed_bootstrap(pool: &PgPool) {
    // Customer 1
    if let Err(e) = seed_customer(pool).await {
        info!(target="server", event="seed_error", step="customer", error=?e, "seed step failed");
    }
    if let Err(e) = seed_provider(pool).await {
        info!(target="server", event="seed_error", step="provider", error=?e, "seed step failed");
    }
    if let Err(e) = seed_demo_conversation(pool).await {
        info!(target="server", event="seed_error", step="conversation", error=?e, "seed step failed");
    }
}

async fn seed_customer(pool: &PgPool) -> sqlx::Result<()> {
    let existing = sqlx::query("SELECT id FROM customers WHERE id = 1")
        .fetch_optional(pool)
        .await?;
    if existing.is_none() {
        sqlx::query("INSERT INTO customers (id, name) VALUES (1, 'Demo Customer')")
            .execute(pool)
            .await?;
        info!(
            target = "server",
            event = "seed_create",
            entity = "customer",
            id = 1,
            "created demo customer"
        );
    }
    Ok(())
}

async fn seed_provider(pool: &PgPool) -> sqlx::Result<()> {
    let existing = sqlx::query("SELECT id FROM providers WHERE id = 1")
        .fetch_optional(pool)
        .await?;
    if existing.is_none() {
        sqlx::query("INSERT INTO providers (id, customer_id, kind, name) VALUES (1, 1, 'sms', 'Mock SMS Provider')")
            .execute(pool)
            .await?;
        info!(
            target = "server",
            event = "seed_create",
            entity = "provider",
            id = 1,
            "created demo provider"
        );
    }
    Ok(())
}

async fn seed_demo_conversation(pool: &PgPool) -> sqlx::Result<()> {
    // Use topic key 'seed:+15550001111:+15550002222:sms'
    let topic = "seed:+15550001111:+15550002222:sms";
    let convo = sqlx::query("SELECT id FROM conversations WHERE topic = $1")
        .bind(topic)
        .fetch_optional(pool)
        .await?;
    let convo_id = if let Some(row) = convo {
        row.get::<i64, _>("id")
    } else {
        let inserted = sqlx::query(
            "INSERT INTO conversations (customer_id, topic) VALUES (1, $1) RETURNING id",
        )
        .bind(topic)
        .fetch_one(pool)
        .await?;
        let id: i64 = inserted.get("id");
        info!(target="server", event="seed_create", entity="conversation", id=id, topic=%topic, "created seed conversation");
        id
    };
    // If no messages yet for this convo, insert one
    let msg_exists = sqlx::query("SELECT id FROM messages WHERE conversation_id = $1 LIMIT 1")
        .bind(convo_id)
        .fetch_optional(pool)
        .await?;
    if msg_exists.is_none() {
        let body_row = sqlx::query("INSERT INTO message_bodies (body) VALUES ($1) RETURNING id")
            .bind("Welcome to the seeded conversation!")
            .fetch_one(pool)
            .await?;
        let body_id: i64 = body_row.get("id");
        let msg_row = sqlx::query("INSERT INTO messages (conversation_id, provider_id, direction, sent_at, received_at, body_id) VALUES ($1, 1, 'inbound', now(), now(), $2) RETURNING id")
            .bind(convo_id)
            .bind(body_id)
            .fetch_one(pool)
            .await?;
        let message_id: i64 = msg_row.get("id");
        // Add one attachment URL
        let att_row = sqlx::query("INSERT INTO attachment_urls (url) VALUES ($1) RETURNING id")
            .bind("https://example.com/demo.jpg")
            .fetch_one(pool)
            .await?;
        let att_id: i64 = att_row.get("id");
        sqlx::query(
            "INSERT INTO message_attachment_urls (message_id, attachment_url_id) VALUES ($1, $2)",
        )
        .bind(message_id)
        .bind(att_id)
        .execute(pool)
        .await?;
        info!(
            target = "server",
            event = "seed_create",
            entity = "message",
            id = message_id,
            convo_id = convo_id,
            "created seed message"
        );
    }
    Ok(())
}
