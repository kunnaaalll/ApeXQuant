use crate::shadow::{LegacyRiskState, RustRiskState};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonState {
    ExactMatch,
    CloseMatch,
    Warning,
    Mismatch,
    Critical,
}

pub struct ComparisonEngine;

impl ComparisonEngine {
    pub fn compare(legacy: &LegacyRiskState, rust: &RustRiskState) -> ComparisonState {
        if legacy.drawdown == rust.drawdown
            && legacy.exposure == rust.exposure
            && legacy.correlation == rust.correlation
            && legacy.hidden_leverage == rust.hidden_leverage
            && legacy.var == rust.var
            && legacy.expected_shortfall == rust.expected_shortfall
            && legacy.circuit_breakers_tripped == rust.circuit_breakers_tripped
            && legacy.recommendation_code == rust.recommendation_code
            && legacy.stress_assessment == rust.stress_assessment
        {
            return ComparisonState::ExactMatch;
        }

        // Qualitative differences are critical mismatches since they determine action logic
        if legacy.circuit_breakers_tripped != rust.circuit_breakers_tripped
            || legacy.recommendation_code != rust.recommendation_code
            || legacy.stress_assessment != rust.stress_assessment
        {
            return ComparisonState::Critical;
        }

        let drawdown_diff = (legacy.drawdown - rust.drawdown).abs();
        let exposure_diff = (legacy.exposure - rust.exposure).abs();
        let correlation_diff = (legacy.correlation - rust.correlation).abs();
        let leverage_diff = (legacy.hidden_leverage - rust.hidden_leverage).abs();
        let var_diff = (legacy.var - rust.var).abs();
        let shortfall_diff = (legacy.expected_shortfall - rust.expected_shortfall).abs();

        let max_diff = drawdown_diff
            .max(exposure_diff)
            .max(correlation_diff)
            .max(leverage_diff)
            .max(var_diff)
            .max(shortfall_diff);

        let close_threshold = Decimal::new(1, 4); // 0.0001
        let warning_threshold = Decimal::new(1, 2); // 0.01
        let mismatch_threshold = Decimal::new(1, 1); // 0.1

        if max_diff <= close_threshold {
            ComparisonState::CloseMatch
        } else if max_diff <= warning_threshold {
            ComparisonState::Warning
        } else if max_diff <= mismatch_threshold {
            ComparisonState::Mismatch
        } else {
            ComparisonState::Critical
        }
    }
}
