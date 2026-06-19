use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidityState {
    Healthy,
    Warning,
    Danger,
    Critical,
    Frozen,
}

pub struct LiquidityCrisisEngine {
    base_depth: Decimal,
}

impl LiquidityCrisisEngine {
    pub fn new(base_depth: Decimal) -> Self {
        Self {
            base_depth: base_depth.max(dec!(0)),
        }
    }

    pub fn apply_crisis(&self, reduction_factor: Decimal) -> Decimal {
        let mut new_depth = self.base_depth * reduction_factor;
        if new_depth < dec!(0) {
            new_depth = dec!(0);
        }
        new_depth
    }

    pub fn evaluate_state(&self, stressed_depth: Decimal) -> LiquidityState {
        if stressed_depth > dec!(1_000_000) {
            LiquidityState::Healthy
        } else if stressed_depth > dec!(500_000) {
            LiquidityState::Warning
        } else if stressed_depth > dec!(100_000) {
            LiquidityState::Danger
        } else if stressed_depth > dec!(10_000) {
            LiquidityState::Critical
        } else {
            LiquidityState::Frozen
        }
    }
}
