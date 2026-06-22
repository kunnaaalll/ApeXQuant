use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonState {
    ExactMatch,
    CloseMatch,
    Warning,
    Mismatch,
    CriticalMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonResult {
    pub order_state: ComparisonState,
    pub fill_quality: ComparisonState,
    pub slippage: ComparisonState,
    pub liquidity_score: ComparisonState,
    pub latency_score: ComparisonState,
    pub microstructure_score: ComparisonState,
    pub execution_risk_score: ComparisonState,
    pub broker_routing_state: ComparisonState,
}

pub struct ComparisonEngine;

impl ComparisonEngine {
    pub const fn new() -> Self {
        Self
    }

    pub fn compare(
        &self,
        shadow_val: Decimal,
        production_val: Decimal,
        thresholds: &crate::shadow::thresholds::ParityThresholds,
    ) -> ComparisonState {
        let diff = (shadow_val - production_val).abs();

        if diff <= thresholds.exact_match {
            ComparisonState::ExactMatch
        } else if diff <= thresholds.close_match {
            ComparisonState::CloseMatch
        } else if diff <= thresholds.warning {
            ComparisonState::Warning
        } else if diff <= thresholds.mismatch {
            ComparisonState::Mismatch
        } else {
            ComparisonState::CriticalMismatch
        }
    }
}
