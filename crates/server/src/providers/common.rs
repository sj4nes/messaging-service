//! Shared provider helpers (Feature 008)

use crate::config::ApiConfig;
use crate::providers::mock::{pick_outcome as pick_outcome_global, Outcome};

/// Choose an outcome for a given logical provider based on per-provider overrides,
/// falling back to global API_* values.
pub fn pick_outcome_for_provider(
    provider: &str,
    cfg: &ApiConfig,
) -> Outcome {
    // For now, delegate to global picker until per-provider cfg is parsed (T008).
    // Later, this will consult cfg.provider_{sms|email}_* fields.
    let _ = provider; // reserved for future use
    pick_outcome_global(cfg)
}
