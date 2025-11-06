use serde::Serialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Serialize)]
pub struct InboundEvent {
    pub event_name: String,
    pub payload: serde_json::Value,
    pub occurred_at: String,
    pub idempotency_key: Option<String>,
    pub source: String, // "api" or "webhook"
}

#[derive(Clone)]
pub struct InboundQueue {
    tx: Sender<InboundEvent>,
}

impl InboundQueue {
    pub fn new(buffer: usize) -> (Self, Receiver<InboundEvent>) {
        let (tx, rx) = channel(buffer);
        (Self { tx }, rx)
    }

    pub async fn enqueue(&self, event: InboundEvent) -> Result<(), String> {
        self.tx
            .send(event)
            .await
            .map_err(|e| format!("queue send failed: {e}"))
    }
}
