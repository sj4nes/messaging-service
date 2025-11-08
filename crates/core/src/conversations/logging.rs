use tracing::{info, warn, error};

use super::upsert::UpsertOutcome;

/// Log a structured event for a conversation upsert outcome tied to a message.
/// direction: inbound | outbound
/// message_id may be 0 if not yet assigned.
pub fn log_upsert_outcome(outcome: &UpsertOutcome, direction: &str, message_id: i64) {
    match outcome {
        UpsertOutcome::Created(id, k) => info!(
            target = "conversation", event = "conversation_created", conversation_id = *id,
            key = %k.key, channel = %k.channel, participant_a = %k.participant_a, participant_b = %k.participant_b,
            direction = direction, message_id = message_id,
            "conversation created"
        ),
        UpsertOutcome::Reused(id, k) => info!(
            target = "conversation", event = "conversation_reused", conversation_id = *id,
            key = %k.key, channel = %k.channel, participant_a = %k.participant_a, participant_b = %k.participant_b,
            direction = direction, message_id = message_id,
            "conversation reused"
        ),
        UpsertOutcome::Failed(err) => {
            error!(
                target = "conversation", event = "conversation_upsert_failed", error = %err,
                direction = direction, message_id = message_id,
                "conversation upsert failed"
            )
        }
    }
}

/// Convenience to log failure before returning an error path in callers.
pub fn log_upsert_failure(context: &str, error: &str) {
    warn!(target="conversation", event="conversation_upsert_failure", context=%context, error=%error, "upsert failure");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversations::{ConversationKey, upsert::UpsertOutcome};

    #[test]
    fn logs_created() {
        let k = ConversationKey { channel: "email".into(), participant_a: "a".into(), participant_b: "b".into(), key: "email:a<->b".into() };
        let outcome = UpsertOutcome::Created(42, k);
        log_upsert_outcome(&outcome, "inbound", 777); // Should not panic
    }
}
