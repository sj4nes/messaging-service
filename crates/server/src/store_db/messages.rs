use anyhow::Result;
use sqlx::PgPool;

use super::normalize::conversation_key;

pub async fn insert_from_inbound(
    _pool: &PgPool,
    _inbound_id: i64,
    channel: &str,
    from: &str,
    to: &str,
    _body: &str,
    _attachments: &[String],
    _timestamp: &str,
) -> Result<()> {
    // Placeholder: compute conversation key for future use; no DB writes yet.
    let _conv_key = conversation_key(channel, from, to);
    Ok(())
}
