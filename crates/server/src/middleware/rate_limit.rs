use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    per_ip_limit_per_min: u32,
    per_sender_limit_per_min: u32,
    inner: Arc<Mutex<Inner>>,
}

struct Counter {
    count: u32,
    window_start: Instant,
}

struct Inner {
    ip: HashMap<String, Counter>,
    sender: HashMap<String, Counter>,
}

impl RateLimiter {
    pub fn new(per_ip: u32, per_sender: u32) -> Self {
        Self {
            per_ip_limit_per_min: per_ip,
            per_sender_limit_per_min: per_sender,
            inner: Arc::new(Mutex::new(Inner {
                ip: HashMap::new(),
                sender: HashMap::new(),
            })),
        }
    }

    fn check_and_inc(counter: &mut HashMap<String, Counter>, key: &str, limit: u32) -> bool {
        let now = Instant::now();
        let entry = counter.entry(key.to_string()).or_insert(Counter {
            count: 0,
            window_start: now,
        });
        if now.duration_since(entry.window_start) >= Duration::from_secs(60) {
            entry.count = 0;
            entry.window_start = now;
        }
        if entry.count < limit {
            entry.count += 1;
            true
        } else {
            false
        }
    }

    pub fn allow_ip(&self, ip: &str) -> bool {
        let mut inner = self.inner.lock().unwrap();
        Self::check_and_inc(&mut inner.ip, ip, self.per_ip_limit_per_min)
    }

    pub fn allow_sender(&self, sender: &str) -> bool {
        let mut inner = self.inner.lock().unwrap();
        Self::check_and_inc(&mut inner.sender, sender, self.per_sender_limit_per_min)
    }
}
