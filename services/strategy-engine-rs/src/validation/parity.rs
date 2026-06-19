use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityState {
    Pass,
    Warning,
    Fail,
}

#[derive(Debug, Clone)]
pub struct ParityResult {
    pub state: ParityState,
    pub differences: Decimal,
}

#[derive(Debug, Clone)]
pub struct ParityValidator;

impl Default for ParityValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ParityValidator {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn verify(
        &self,
        strategy_health: Decimal,
        confidence: Decimal,
        allocation: Decimal,
        recommendation: Decimal,
        context_score: Decimal,
        drift_score: Decimal,
        reference_health: Decimal,
        reference_confidence: Decimal,
        reference_allocation: Decimal,
        reference_recommendation: Decimal,
        reference_context_score: Decimal,
        reference_drift_score: Decimal,
    ) -> ParityResult {
        let diff = (strategy_health - reference_health).abs()
            + (confidence - reference_confidence).abs()
            + (allocation - reference_allocation).abs()
            + (recommendation - reference_recommendation).abs()
            + (context_score - reference_context_score).abs()
            + (drift_score - reference_drift_score).abs();

        let state = if diff == Decimal::ZERO {
            ParityState::Pass
        } else if diff <= Decimal::new(5, 2) {
            ParityState::Warning
        } else {
            ParityState::Fail
        };

        ParityResult {
            state,
            differences: diff,
        }
    }
}
