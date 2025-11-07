//! Provider registry and trait (Feature 008 - Phase 2)
//!
//! Defines:
//! - Provider trait with name + dispatch
//! - OutboundMessage & DispatchResult internal types
//! - ProviderRegistry container (channel → provider mapping)
//!
//! Actual provider implementations wired in later phases (US1).

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::ApiConfig;
use crate::providers::mock::Outcome;

/// Channel type supported for outbound messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    Sms,
    Mms,
    Email,
}

impl ChannelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelKind::Sms => "sms",
            ChannelKind::Mms => "mms",
            ChannelKind::Email => "email",
        }
    }
}

impl std::str::FromStr for ChannelKind {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sms" => Ok(ChannelKind::Sms),
            "mms" => Ok(ChannelKind::Mms),
            "email" => Ok(ChannelKind::Email),
            _ => Err("unsupported channel"),
        }
    }
}

/// Internal representation of an outbound message ready for provider dispatch.
///
/// Contract (US1):
/// - `channel` must map to an initialized provider; otherwise routing layer records `invalid_routing`.
/// - `to`, `from`, `body` may be empty strings (validation handled earlier in API layer in future phases).
/// - `attachments` currently unused by mock providers but retained for parity with future real providers.
/// - `idempotency_key` propagates idempotency semantics into provider layer for future dedup.
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub channel: ChannelKind,
    pub to: String,
    pub from: String,
    pub body: String,
    pub attachments: Vec<String>,
    pub idempotency_key: Option<String>,
}

/// Result of a provider dispatch attempt.
///
/// Contract:
/// - `provider_name` echoes the logical provider (metrics/log correlation key).
/// - `outcome` drives metrics & breaker state updates.
#[derive(Debug, Clone)]
pub struct DispatchResult {
    pub provider_name: String,
    pub outcome: Outcome,
}

/// Provider abstraction; implementations will perform mock/real dispatch.
///
/// Implementor guidance:
/// - Must be cheap to clone via Arc (no large interior mutable state; use separate stores if needed).
/// - `dispatch` must be side-effect free except for external I/O (mock providers simulate outcomes only).
/// - Deterministic test support: when seeds provided, outcome sequence must be reproducible.
/// - Error modes: return `Outcome::Error` for server-side failures; `Outcome::Timeout` for simulated timeouts.
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn dispatch(&self, msg: &OutboundMessage, cfg: &ApiConfig) -> DispatchResult;
}

/// Provider registry mapping channel → provider instance.
#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<ChannelKind, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    pub fn insert(&mut self, channel: ChannelKind, provider: Arc<dyn Provider>) {
        self.providers.insert(channel, provider);
    }
    pub fn get(&self, channel: ChannelKind) -> Option<&Arc<dyn Provider>> {
        self.providers.get(&channel)
    }
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Clone for ProviderRegistry {
    fn clone(&self) -> Self {
        // Arc<dyn Provider> is cheap to clone; clone the map entries.
        let mut new_map: HashMap<ChannelKind, Arc<dyn Provider>> = HashMap::new();
        for (k, v) in self.providers.iter() {
            new_map.insert(*k, v.clone());
        }
        ProviderRegistry { providers: new_map }
    }
}
