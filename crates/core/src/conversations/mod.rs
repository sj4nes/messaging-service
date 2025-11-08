//! Conversations module: normalization and key derivation scaffolding.

pub mod key;
pub mod logging;
pub mod metrics;
pub mod normalize_email;
pub mod normalize_phone;
pub mod upsert;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationKey {
    pub channel: String,
    pub participant_a: String,
    pub participant_b: String,
    pub key: String,
}
