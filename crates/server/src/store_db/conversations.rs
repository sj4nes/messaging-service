// Placeholder DB conversation/message listing until full mapping implemented.
use anyhow::Result;
use sqlx::PgPool;

#[derive(Debug, serde::Serialize)]
pub struct ConversationRow {
    pub id: i64,
    pub topic: Option<String>,
    pub created_at: String,
}

pub async fn list_conversations(_pool: &PgPool, _limit: i64) -> Result<Vec<ConversationRow>> {
    // TODO: implement query joining participants/messages for summary
    Ok(Vec::new())
}

#[derive(Debug, serde::Serialize)]
pub struct MessageRow {
    pub id: i64,
    pub direction: String,
    pub body: String,
    pub sent_at: String,
    pub received_at: Option<String>,
}

pub async fn list_messages(
    _pool: &PgPool,
    _conversation_id: i64,
    _limit: i64,
) -> Result<Vec<MessageRow>> {
    // TODO: implement query
    Ok(Vec::new())
}
