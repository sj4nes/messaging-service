use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use sqlx::{PgPool, Row};
use tracing::{instrument, warn};
use twox_hash::xxh3::hash64;

use super::normalize::conversation_key; // legacy helper (to be removed)
use crate::logging::message_persisted;
use messaging_core::conversations::{
    key::ChannelKind,
    logging::log_upsert_outcome,
    metrics::metrics,
    upsert::{upsert_conversation, UpsertOutcome},
};
use tracing::info;

/// Persist an inbound message using simplified bootstrap rules:
/// - Assumes a single bootstrap customer (id=1) and provider (id=1) already exist (future migration may ensure this)
/// - Upserts a conversation keyed by normalized endpoints+channel (temporary table-less approach: search existing messages for latest conversation id matching provider+participants)
#[instrument(skip(pool, body, attachments))]
pub async fn insert_from_inbound(
    pool: &PgPool,
    channel: &str,
    from: &str,
    to: &str,
    body: &str,
    attachments: &[String],
    timestamp: &str,
) -> Result<i64> {
    // Map channel string to ChannelKind
    let channel_kind = match channel {
        "email" => ChannelKind::Email,
        "sms" => ChannelKind::Sms,
        "mms" => ChannelKind::Mms,
        other => return Err(anyhow::anyhow!(format!("unsupported channel: {other}"))),
    };
    // Parse timestamp
    let ts: DateTime<Utc> = timestamp.parse().unwrap_or_else(|_| Utc::now());
    // Insert body row if present
    let body_id: Option<i64> = if body.is_empty() {
        None
    } else {
        // Attempt insert with ON CONFLICT to deduplicate.
        // If a duplicate exists we won't get a RETURNING row, so fallback to SELECT.
        let inserted = sqlx::query(
            r#"INSERT INTO message_bodies (body) VALUES ($1)
            ON CONFLICT (body) DO NOTHING RETURNING id"#,
        )
        .bind(body)
        .fetch_optional(pool)
        .await?;
        if let Some(row) = inserted {
            Some(row.get("id"))
        } else {
            let existing = sqlx::query(r#"SELECT id FROM message_bodies WHERE body = $1 LIMIT 1"#)
                .bind(body)
                .fetch_one(pool)
                .await?;
            Some(existing.get("id"))
        }
    };
    // Upsert conversation using normalized endpoints
    let upsert_outcome = upsert_conversation(pool, channel_kind.clone(), from, to, ts).await;
    let (convo_id, conv_key_str) = match &upsert_outcome {
        UpsertOutcome::Created(id, k) => {
            metrics().inc_created();
            (*id, k.key.clone())
        }
        UpsertOutcome::Reused(id, k) => {
            metrics().inc_reused();
            (*id, k.key.clone())
        }
        UpsertOutcome::Failed(err) => {
            metrics().inc_failures();
            return Err(anyhow::anyhow!(err.clone()));
        }
    };
    // Idempotency: check for existing message with same convo + direction + sent_at + body_id
    if let Some(row) = sqlx::query(
        r#"SELECT id FROM messages WHERE conversation_id = $1 AND direction = 'inbound' AND sent_at = $2 AND body_id IS NOT DISTINCT FROM $3 LIMIT 1"#,
    )
    .bind(convo_id)
    .bind(ts)
    .bind(body_id)
    .fetch_optional(pool)
    .await? {
        let existing_id: i64 = row.get("id");
        log_upsert_outcome(&upsert_outcome, "inbound", existing_id);
        return Ok(existing_id);
    }
    // Insert message referencing body
    let rec = sqlx::query(
        r#"INSERT INTO messages (conversation_id, provider_id, direction, sent_at, received_at, body_id)
           VALUES ($1, 1, 'inbound', $2, $2, $3) RETURNING id"#,
    )
    .bind(convo_id)
    .bind(ts)
    .bind(body_id)
    .fetch_one(pool)
    .await?;
    let message_id: i64 = rec.get("id");
    // Increment message_count and update last_activity_at atomically post-insert
    let _ = sqlx::query(
        "UPDATE conversations SET message_count = message_count + 1, last_activity_at = GREATEST(last_activity_at, $2) WHERE id = $1",
    )
    .bind(convo_id)
    .bind(ts)
    .execute(pool)
    .await;
    log_upsert_outcome(&upsert_outcome, "inbound", message_id);
    message_persisted("inbound_persisted", message_id, convo_id, &conv_key_str);
    // Persist attachments (URLs) if any
    for url in attachments {
        if let Err(e) = persist_attachment(pool, message_id, url).await {
            warn!(target="server", event="attach_persist_fail", error=?e, message_id, url=%url, "failed to persist/link attachment; continuing");
        }
    }
    Ok(message_id)
}

/// Persist an outbound message (API initiated) using same rules as inbound for now.
/// Differences:
/// - direction = 'outbound'
/// - provider_id currently hard-coded to 1 (bootstrap mock provider)
/// - body stored/deduplicated identically via message_bodies table
#[instrument(skip(pool, body, attachments))]
pub async fn insert_outbound(
    pool: &PgPool,
    channel: &str,
    from: &str,
    to: &str,
    body: &str,
    attachments: &[String],
    timestamp: &str,
) -> Result<i64> {
    let channel_kind = match channel {
        "email" => ChannelKind::Email,
        "sms" => ChannelKind::Sms,
        "mms" => ChannelKind::Mms,
        other => return Err(anyhow::anyhow!(format!("unsupported channel: {other}"))),
    };
    let ts: DateTime<Utc> = timestamp.parse().unwrap_or_else(|_| Utc::now());
    let body_id: Option<i64> = if body.is_empty() {
        None
    } else {
        let inserted = sqlx::query(
            r#"INSERT INTO message_bodies (body) VALUES ($1)
            ON CONFLICT (body) DO NOTHING RETURNING id"#,
        )
        .bind(body)
        .fetch_optional(pool)
        .await?;
        if let Some(row) = inserted {
            Some(row.get("id"))
        } else {
            let existing = sqlx::query(r#"SELECT id FROM message_bodies WHERE body = $1 LIMIT 1"#)
                .bind(body)
                .fetch_one(pool)
                .await?;
            Some(existing.get("id"))
        }
    };
    let upsert_outcome = upsert_conversation(pool, channel_kind.clone(), from, to, ts).await;
    let (convo_id, conv_key_str) = match &upsert_outcome {
        UpsertOutcome::Created(id, k) => {
            metrics().inc_created();
            (*id, k.key.clone())
        }
        UpsertOutcome::Reused(id, k) => {
            metrics().inc_reused();
            (*id, k.key.clone())
        }
        UpsertOutcome::Failed(err) => {
            metrics().inc_failures();
            return Err(anyhow::anyhow!(err.clone()));
        }
    };
    if let Some(row) = sqlx::query(
        r#"SELECT id FROM messages WHERE conversation_id = $1 AND direction = 'outbound' AND sent_at = $2 AND body_id IS NOT DISTINCT FROM $3 LIMIT 1"#,
    )
    .bind(convo_id)
    .bind(ts)
    .bind(body_id)
    .fetch_optional(pool)
    .await? {
        let existing_id: i64 = row.get("id");
        log_upsert_outcome(&upsert_outcome, "outbound", existing_id);
        return Ok(existing_id);
    }
    let rec = sqlx::query(
        r#"INSERT INTO messages (conversation_id, provider_id, direction, sent_at, received_at, body_id)
           VALUES ($1, 1, 'outbound', $2, $2, $3) RETURNING id"#,
    )
    .bind(convo_id)
    .bind(ts)
    .bind(body_id)
    .fetch_one(pool)
    .await?;
    let message_id: i64 = rec.get("id");
    let _ = sqlx::query(
        "UPDATE conversations SET message_count = message_count + 1, last_activity_at = GREATEST(last_activity_at, $2) WHERE id = $1",
    )
    .bind(convo_id)
    .bind(ts)
    .execute(pool)
    .await;
    log_upsert_outcome(&upsert_outcome, "outbound", message_id);
    message_persisted("outbound_persisted", message_id, convo_id, &conv_key_str);
    for url in attachments {
        if let Err(e) = persist_attachment(pool, message_id, url).await {
            warn!(target="server", event="attach_persist_fail", error=?e, message_id, url=%url, "failed to persist/link attachment (outbound); continuing");
        }
    }
    Ok(message_id)
}

// Legacy ensure_conversation removed; durable conversations now handled by messaging_core::conversations::upsert_conversation.

// Runtime detection for attachment_urls schema variants
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum AttachmentSchema {
    UrlOnly,    // columns: id, url
    RawHash,    // columns: id, raw, hash
    RawHashUrl, // columns: id, raw, hash, url
}

static ATT_SCHEMA: OnceCell<AttachmentSchema> = OnceCell::new();

async fn detect_attachment_schema(pool: &PgPool) -> AttachmentSchema {
    if let Some(v) = ATT_SCHEMA.get() {
        return *v;
    }
    // Inspect information_schema.columns for attachment_urls
    let rows = match sqlx::query(
        r#"SELECT column_name FROM information_schema.columns WHERE table_name='attachment_urls'"#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => {
            let _ = ATT_SCHEMA.set(AttachmentSchema::UrlOnly);
            return AttachmentSchema::UrlOnly;
        }
    };
    let mut has_url = false;
    let mut has_raw = false;
    let mut has_hash = false;
    for r in rows {
        let c: String = r.get("column_name");
        match c.as_str() {
            "url" => has_url = true,
            "raw" => has_raw = true,
            "hash" => has_hash = true,
            _ => {}
        }
    }
    let detected = if has_raw && has_hash && has_url {
        AttachmentSchema::RawHashUrl
    } else if has_raw && has_hash {
        AttachmentSchema::RawHash
    } else {
        AttachmentSchema::UrlOnly
    };
    let _ = ATT_SCHEMA.set(detected);
    detected
}

async fn persist_attachment(pool: &PgPool, message_id: i64, url: &str) -> Result<()> {
    let schema = detect_attachment_schema(pool).await;
    let att_id: Option<i64> = match schema {
        AttachmentSchema::UrlOnly => {
            // Upsert by url
            if let Some(row) = sqlx::query(
                r#"INSERT INTO attachment_urls (url) VALUES ($1)
                    ON CONFLICT (url) DO NOTHING RETURNING id"#,
            )
            .bind(url)
            .fetch_optional(pool)
            .await?
            {
                Some(row.get("id"))
            } else {
                // lookup existing
                match sqlx::query(r#"SELECT id FROM attachment_urls WHERE url = $1 LIMIT 1"#)
                    .bind(url)
                    .fetch_one(pool)
                    .await
                {
                    Ok(r) => Some(r.get("id")),
                    Err(_) => None,
                }
            }
        }
        AttachmentSchema::RawHash | AttachmentSchema::RawHashUrl => {
            let hash = hash64(url.as_bytes()) as i64;
            // If RawHashUrl, also populate url if column exists
            let insert_sql = match schema {
                AttachmentSchema::RawHashUrl => {
                    r#"INSERT INTO attachment_urls (raw, hash, url) VALUES ($1, $2, $1)
                    ON CONFLICT (url) DO NOTHING RETURNING id"#
                }
                _ => {
                    r#"INSERT INTO attachment_urls (raw, hash) VALUES ($1, $2)
                    ON CONFLICT (hash) DO NOTHING RETURNING id"#
                }
            };
            if let Some(row) = sqlx::query(insert_sql)
                .bind(url)
                .bind(hash)
                .fetch_optional(pool)
                .await?
            {
                Some(row.get("id"))
            } else {
                match schema {
                    AttachmentSchema::RawHashUrl => {
                        match sqlx::query(
                            r#"SELECT id FROM attachment_urls WHERE url = $1 LIMIT 1"#,
                        )
                        .bind(url)
                        .fetch_one(pool)
                        .await
                        {
                            Ok(r) => Some(r.get("id")),
                            Err(_) => None,
                        }
                    }
                    AttachmentSchema::RawHash => {
                        // Lookup by hash then verify raw matches to detect collisions.
                        match sqlx::query(
                            r#"SELECT id, raw FROM attachment_urls WHERE hash = $1 LIMIT 1"#,
                        )
                        .bind(hash)
                        .fetch_one(pool)
                        .await
                        {
                            Ok(r) => {
                                let existing_raw: String = r.get("raw");
                                if existing_raw != url {
                                    warn!(target="server", event="attach_hash_collision", message_id, existing_raw=%existing_raw, url=%url, hash=%hash, "hash collision on legacy RawHash schema; skipping link");
                                    None
                                } else {
                                    Some(r.get("id"))
                                }
                            }
                            Err(_) => None,
                        }
                    }
                    AttachmentSchema::UrlOnly => None,
                }
            }
        }
    };

    if let Some(id) = att_id {
        sqlx::query(
            r#"INSERT INTO message_attachment_urls (message_id, attachment_url_id) VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
        )
        .bind(message_id)
        .bind(id)
        .execute(pool)
        .await?;
    }
    Ok(())
}
