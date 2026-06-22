use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::circuit_breaker::ExecutionProtectionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidityRegime {
    Excellent,
    Normal,
    Weak,
    Poor,
    Broken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiquidityGuards {
    pub book_depth: Decimal,
    pub spread_quality: Decimal,
    pub imbalance: Decimal,
    pub available_liquidity: Decimal,
}

impl LiquidityGuards {
    pub fn new(
        book_depth: Decimal,
        spread_quality: Decimal,
        imbalance: Decimal,
        available_liquidity: Decimal,
    ) -> Self {
        Self {
            book_depth,
            spread_quality,
            imbalance,
            available_liquidity,
        }
    }

    pub fn get_regime(&self) -> LiquidityRegime {
        // Simple heuristic: If available liquidity is very low or imbalance is extreme
        if self.available_liquidity <= dec!(0.1) || self.imbalance >= dec!(0.9) {
            return LiquidityRegime::Broken;
        }

        if self.available_liquidity <= dec!(1.0) || self.imbalance >= dec!(0.7) {
            return LiquidityRegime::Poor;
        }

        if self.available_liquidity <= dec!(5.0) || self.imbalance >= dec!(0.5) {
            return LiquidityRegime::Weak;
        }

        if self.available_liquidity >= dec!(20.0) && self.imbalance <= dec!(0.2) {
            return LiquidityRegime::Excellent;
        }

        LiquidityRegime::Normal
    }

    pub fn get_protection_state(&self) -> ExecutionProtectionState {
        match self.get_regime() {
            LiquidityRegime::Excellent | LiquidityRegime::Normal => ExecutionProtectionState::Normal,
            LiquidityRegime::Weak => ExecutionProtectionState::Warning,
            LiquidityRegime::Poor => ExecutionProtectionState::Restricted,
            LiquidityRegime::Broken => {
                if self.available_liquidity <= dec!(0.01) {
                    ExecutionProtectionState::Frozen
                } else {
                    ExecutionProtectionState::Critical
                }
            }
        }
    }
}
