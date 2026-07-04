use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverfitDetector {
    pub complexity_penalty_threshold: Decimal,
}

impl OverfitDetector {
    pub fn new() -> Self {
        Self {
            complexity_penalty_threshold: dec!(0.15),
        }
    }

    /// Calculate overfit score: in_sample / out_of_sample.
    /// If out_of_sample is zero/negative, it's highly overfit (returns high ratio).
    pub fn check_overfit(
        &self,
        in_sample_sharpe: Decimal,
        out_of_sample_sharpe: Decimal,
        complexity_degree: u32,
    ) -> Decimal {
        if out_of_sample_sharpe <= Decimal::ZERO {
            return dec!(99.0); // Extreme overfit
        }

        let base_ratio = in_sample_sharpe / out_of_sample_sharpe;
        let penalty = Decimal::from(complexity_degree) * self.complexity_penalty_threshold;
        base_ratio + penalty
    }
}

impl Default for OverfitDetector {
    fn default() -> Self {
        Self::new()
    }
}
