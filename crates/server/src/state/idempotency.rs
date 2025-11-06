use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct IdempotencyStore {
    inner: std::sync::Arc<Mutex<HashMap<String, Instant>>>,
    ttl: Duration,
}

impl IdempotencyStore {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            inner: std::sync::Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Returns true if key is new (and stores it). Returns false if key was seen recently.
    pub fn seen_or_insert(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut m = self.inner.lock().unwrap();
        // Drop expired
        m.retain(|_, &mut t| now.duration_since(t) < self.ttl);
        if let Some(t) = m.get(key) {
            if now.duration_since(*t) < self.ttl {
                return false;
            }
        }
        m.insert(key.to_string(), now);
        true
    }
}
