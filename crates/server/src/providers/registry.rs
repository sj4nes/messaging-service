//! Provider registry and trait scaffolding (Feature 008 - Phase 1/2)
//!
//! Defines the `Provider` trait and a simple registry structure to be populated in Phase 2.
//! Phase 1 adds the file; Phase 2 will fill in trait methods and data structures.

#[allow(dead_code)]
pub trait Provider {
    /// Provider human-readable name (e.g., "sms-mms", "email")
    fn name(&self) -> &str;
    // Phase 2: dispatch method signature will be added.
}

#[allow(dead_code)]
pub struct ProviderRegistry {
    // Phase 2: mapping channel -> provider instance refs
}

impl ProviderRegistry {
    #[allow(dead_code)]
    pub fn new() -> Self { Self { } }
}
