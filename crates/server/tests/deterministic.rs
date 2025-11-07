// Feature 008 - US3 Deterministic unit test (T031)
// Verifies that with a fixed seed, predicted outcome distribution matches actual rolls.
use messaging_server::config::ApiConfig;
use messaging_server::providers::common::{
    init_rng_seeds, pick_outcome_for_provider, predict_outcomes_from_seed, seed_provider_rng,
};
use messaging_server::providers::mock::Outcome;

#[test]
fn sms_provider_deterministic_sequence_matches_prediction() {
    // Config: error 10%, rate limit 5%, timeout 0% to simplify
    let mut cfg = ApiConfig::default();
    cfg.provider_sms_error_pct = Some(10); // 10 errors
    cfg.provider_sms_ratelimit_pct = Some(5); // 5 rate limits
    cfg.provider_sms_timeout_pct = Some(0); // 0 timeouts
    cfg.provider_sms_seed = Some(12345);
    init_rng_seeds(&cfg);
    seed_provider_rng("sms-mms", 12345);

    // Predict outcomes for first N rolls from the seed
    let n = 50usize;
    let (succ_p, rate_p, err_p, to_p) = predict_outcomes_from_seed("sms-mms", &cfg, 12345, n);
    // Collect actual outcomes
    let mut succ_a = 0u32;
    let mut rate_a = 0u32;
    let mut err_a = 0u32;
    let mut to_a = 0u32;
    for _ in 0..n {
        let (outcome, _roll) = pick_outcome_for_provider("sms-mms", &cfg);
        match outcome {
            Outcome::Success => succ_a += 1,
            Outcome::RateLimited => rate_a += 1,
            Outcome::Error => err_a += 1,
            Outcome::Timeout => to_a += 1,
        }
    }
    assert_eq!(succ_a, succ_p, "success count should match prediction");
    assert_eq!(rate_a, rate_p, "ratelimit count should match prediction");
    assert_eq!(err_a, err_p, "error count should match prediction");
    assert_eq!(to_a, to_p, "timeout count should match prediction");
}
