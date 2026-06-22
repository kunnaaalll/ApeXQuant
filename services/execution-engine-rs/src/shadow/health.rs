#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowHealth {
    Excellent,
    Good,
    Normal,
    Weak,
    Critical,
}

pub struct HealthEngine;

impl HealthEngine {
    pub fn evaluate(
        parity_score: &crate::shadow::parity::ParityScore,
        validator: &crate::shadow::validator::GoLiveValidator,
    ) -> ShadowHealth {
        let score = parity_score.value;
        use rust_decimal_macros::dec;

        let base_health = if score >= dec!(99.0) {
            ShadowHealth::Excellent
        } else if score >= dec!(95.0) {
            ShadowHealth::Good
        } else if score >= dec!(90.0) {
            ShadowHealth::Normal
        } else if score >= dec!(80.0) {
            ShadowHealth::Weak
        } else {
            ShadowHealth::Critical
        };

        // If the validator states we are dropping streaks heavily, we might cap health.
        // For zero panics and determinism, we keep the mapping strict to the parity score
        // but verify streaks.
        if validator.consecutive_parity_streaks == 0 && base_health == ShadowHealth::Excellent {
            return ShadowHealth::Good;
        }

        base_health
    }
}
