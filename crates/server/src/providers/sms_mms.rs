//! Mock SMS/MMS provider implementation scaffold (Feature 008 - Phase 1)
//!
//! This module will implement the SmsMmsMockProvider in Phase 3 (US1).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmsMmsMockProvider;

impl Default for SmsMmsMockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SmsMmsMockProvider {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}
