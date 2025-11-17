use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct ConversationMetrics {
    pub created: AtomicU64,
    pub reused: AtomicU64,
    pub failures: AtomicU64,
}

impl Default for ConversationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationMetrics {
    pub const fn new() -> Self {
        Self {
            created: AtomicU64::new(0),
            reused: AtomicU64::new(0),
            failures: AtomicU64::new(0),
        }
    }
    pub fn inc_created(&self) {
        self.created.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_reused(&self) {
        self.reused.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_failures(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
    }
}

static METRICS: ConversationMetrics = ConversationMetrics::new();

pub fn metrics() -> &'static ConversationMetrics {
    &METRICS
}
