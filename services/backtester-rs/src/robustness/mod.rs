//! Robustness Validation Module
//!
//! Evaluates how strategies perform under execution degradation including
//! variations in spread, latency, and slippage.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct DegradationProfile {
    pub additional_spread_ticks: u32,
    pub latency_ms: i64,
    pub slippage_ticks: u32,
}

#[derive(Debug, Clone)]
pub struct RobustnessEvaluation {
    pub passes: bool,
    pub breakdown_score: Decimal,
}

pub struct RobustnessEvaluator;

impl RobustnessEvaluator {
    pub fn evaluate(_strategy_id: &str, profile: &DegradationProfile) -> Result<RobustnessEvaluation, &'static str> {
        let mut score = Decimal::from(1); // Start at 1.0 (100%)

        // Penalize spread: 2.5% per additional spread tick
        score -= Decimal::from(profile.additional_spread_ticks) * Decimal::new(25, 3);

        // Penalize slippage: 5.0% per slippage tick
        score -= Decimal::from(profile.slippage_ticks) * Decimal::new(5, 2);

        // Penalize latency: 0.1% per ms of latency
        score -= Decimal::from(profile.latency_ms) * Decimal::new(1, 3);

        // Bound between 0.0 and 1.0
        if score < Decimal::ZERO {
            score = Decimal::ZERO;
        }

        let passes = score >= Decimal::new(60, 2); // passes if score is >= 60%

        Ok(RobustnessEvaluation {
            passes,
            breakdown_score: score,
        })
    }
}
