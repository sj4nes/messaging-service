// Feature 008 - US1: Unit test for provider registry mapping & routing (T020)
use messaging_server::config::ApiConfig;
use messaging_server::providers::mock::Outcome;
use messaging_server::providers::registry::{
    ChannelKind, DispatchResult, OutboundMessage, Provider, ProviderRegistry,
};
use std::sync::Arc;

struct DummyProvider(&'static str);
impl Provider for DummyProvider {
    fn name(&self) -> &str {
        self.0
    }
    fn dispatch(&self, _msg: &OutboundMessage, _cfg: &ApiConfig) -> DispatchResult {
        DispatchResult {
            provider_name: self.0.to_string(),
            outcome: Outcome::Success,
        }
    }
}

#[test]
fn registry_insert_and_lookup() {
    let mut reg = ProviderRegistry::new();
    reg.insert(ChannelKind::Sms, Arc::new(DummyProvider("sms-mms")));
    reg.insert(ChannelKind::Email, Arc::new(DummyProvider("email")));
    assert!(reg.get(ChannelKind::Sms).is_some());
    assert!(reg.get(ChannelKind::Email).is_some());
    assert!(
        reg.get(ChannelKind::Mms).is_none(),
        "no dedicated mms provider yet (should route via sms-mms later)"
    );
}
