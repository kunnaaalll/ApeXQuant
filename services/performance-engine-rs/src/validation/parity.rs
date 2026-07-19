use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityResult {
    pub agreement_percentage: Decimal,
    pub passed: bool,
}

pub struct PerformanceParityValidator;

impl PerformanceParityValidator {
    pub fn new() -> Self {
        Self
    }

    /// Verifies that the Rust engine and TypeScript engine achieve parity.
    pub fn validate_parity(
        &self,
        shadow_agreement_percentage: Decimal,
        required_threshold: Decimal,
    ) -> ParityResult {
        let passed = shadow_agreement_percentage >= required_threshold;

        ParityResult {
            agreement_percentage: shadow_agreement_percentage,
            passed,
        }
    }
}

impl Default for PerformanceParityValidator {
    fn default() -> Self {
        Self::new()
    }
}
