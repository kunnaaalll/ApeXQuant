use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationRecommendation {
    Increase,
    Reduce,
    Maintain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationTarget {
    Symbol(String),
    Timeframe(String),
    Session(String),
    Risk(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub target: OptimizationTarget,
    pub action: AllocationRecommendation,
    pub magnitude: Decimal, // E.g. percentage to change
    pub reason: String,
}

pub struct Optimizer;

impl Optimizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer {
    pub fn optimize_allocation(
        &self,
        target: OptimizationTarget,
        current_win_rate: Decimal,
        confidence: Decimal,
        regime_match: Decimal,
    ) -> OptimizationResult {
        let seventy = Decimal::new(70, 0);
        let fifty = Decimal::new(50, 0);
        let eighty = Decimal::new(80, 0);

        let action =
            if current_win_rate >= seventy && confidence >= eighty && regime_match >= eighty {
                AllocationRecommendation::Increase
            } else if current_win_rate < fifty || confidence < fifty || regime_match < fifty {
                AllocationRecommendation::Reduce
            } else {
                AllocationRecommendation::Maintain
            };

        let magnitude = match action {
            AllocationRecommendation::Increase => Decimal::new(15, 0),
            AllocationRecommendation::Reduce => Decimal::new(15, 0),
            AllocationRecommendation::Maintain => Decimal::new(0, 0),
        };

        OptimizationResult {
            target,
            action,
            magnitude,
            reason: format!(
                "Win Rate: {}%, Confidence: {}, Regime Match: {}",
                current_win_rate, confidence, regime_match
            ),
        }
    }
}
