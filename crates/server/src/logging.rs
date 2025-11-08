use tracing::info;

pub fn message_persisted(event: &str, message_id: i64, conversation_id: i64, key: &str) {
    info!(target="server", event=%event, message_id, conversation_id, key=%key, "message persisted");
}
