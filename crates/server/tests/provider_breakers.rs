// Feature 008 - US2 Unit test (T026): ProviderBreakers isolation
use messaging_server::middleware::circuit_breaker::CircuitBreaker;
use messaging_server::state::breakers::ProviderBreakers;

#[test]
fn provider_breakers_hold_distinct_instances() {
    let mut map = std::collections::HashMap::new();
    map.insert("sms-mms".to_string(), CircuitBreaker::new(1, 5));
    map.insert("email".to_string(), CircuitBreaker::new(1, 5));
    let pb = ProviderBreakers::new(map);
    assert!(!pb.is_empty(), "expected non-empty provider breakers map");
    let sms_ptr = pb.get("sms-mms").expect("sms breaker") as *const _;
    let email_ptr = pb.get("email").expect("email breaker") as *const _;
    assert_ne!(
        sms_ptr, email_ptr,
        "breakers should be distinct instances (isolation)"
    );
}
