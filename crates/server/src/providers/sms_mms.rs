//! Mock SMS/MMS provider implementation (Feature 008 - US1)
//! Combines SMS + MMS under single logical provider.

use crate::config::ApiConfig;
use crate::providers::common::pick_outcome_for_provider;
use crate::providers::mock::Outcome;
use crate::providers::registry::{DispatchResult, OutboundMessage, Provider};

#[derive(Debug, Clone)]
pub struct SmsMmsMockProvider;

impl Default for SmsMmsMockProvider {
    fn default() -> Self {
        Self
    }
}

impl SmsMmsMockProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for SmsMmsMockProvider {
    fn name(&self) -> &str {
        "sms-mms"
    }
    fn dispatch(&self, _msg: &OutboundMessage, cfg: &ApiConfig) -> DispatchResult {
        let outcome: Outcome = pick_outcome_for_provider(self.name(), cfg);
        DispatchResult {
            provider_name: self.name().to_string(),
            outcome,
        }
    }
}
