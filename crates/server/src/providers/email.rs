//! Mock Email provider implementation (Feature 008 - US1)

use crate::config::ApiConfig;
use crate::providers::common::pick_outcome_for_provider;
use crate::providers::mock::Outcome;
use crate::providers::registry::{DispatchResult, OutboundMessage, Provider};

#[derive(Debug, Clone)]
pub struct EmailMockProvider;

impl Default for EmailMockProvider {
    fn default() -> Self {
        Self
    }
}

impl EmailMockProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for EmailMockProvider {
    fn name(&self) -> &str {
        "email"
    }
    fn dispatch(&self, _msg: &OutboundMessage, cfg: &ApiConfig) -> DispatchResult {
        let (outcome, _roll): (Outcome, u32) = pick_outcome_for_provider(self.name(), cfg);
        DispatchResult {
            provider_name: self.name().to_string(),
            outcome,
        }
    }
}
