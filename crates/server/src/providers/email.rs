//! Mock Email provider implementation scaffold (Feature 008 - Phase 1)
//!
//! This module will implement the EmailMockProvider in Phase 3 (US1).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmailMockProvider;

impl Default for EmailMockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailMockProvider {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}
