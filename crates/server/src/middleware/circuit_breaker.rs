use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone)]
pub struct CircuitBreaker {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    failures: u32,
    state: BreakerState,
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout_secs: u64) -> Self {
        Self {
            failure_threshold,
            recovery_timeout: Duration::from_secs(recovery_timeout_secs),
            inner: Arc::new(Mutex::new(Inner {
                failures: 0,
                state: BreakerState::Closed,
                opened_at: None,
            })),
        }
    }

    pub fn state(&self) -> BreakerState {
        let inner = self.inner.lock().unwrap();
        inner.state
    }

    pub fn before_request(&self) -> BreakerState {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            BreakerState::Open => {
                if let Some(opened) = inner.opened_at {
                    if opened.elapsed() >= self.recovery_timeout {
                        inner.state = BreakerState::HalfOpen;
                    }
                }
                inner.state
            }
            _ => inner.state,
        }
    }

    pub fn record_success(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.failures = 0;
        inner.state = BreakerState::Closed;
        inner.opened_at = None;
    }

    pub fn record_failure(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.failures += 1;
        if inner.failures >= self.failure_threshold {
            inner.state = BreakerState::Open;
            inner.opened_at = Some(Instant::now());
        }
    }
}
