//! Per-provider circuit breaker storage (Feature 008 - T010)
//!
//! Provides a lightweight mapping from provider name to individual `CircuitBreaker` instances.
//! These will be used in User Story 2 to isolate failures.

use std::collections::HashMap;
use std::sync::Arc;

use crate::middleware::circuit_breaker::CircuitBreaker;

#[derive(Clone, Default)]
pub struct ProviderBreakers {
    inner: Arc<HashMap<String, CircuitBreaker>>, // immutable map after construction
}

impl ProviderBreakers {
    pub fn new(map: HashMap<String, CircuitBreaker>) -> Self {
        Self {
            inner: Arc::new(map),
        }
    }
    pub fn get(&self, name: &str) -> Option<&CircuitBreaker> {
        self.inner.get(name)
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
