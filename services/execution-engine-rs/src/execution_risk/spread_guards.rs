use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

use super::circuit_breaker::ExecutionProtectionState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpreadGuards {
    pub current_spread: Decimal,
    pub average_spread: Decimal,
    pub spread_multiplier: Decimal,
}

impl SpreadGuards {
    pub fn new(current_spread: Decimal, average_spread: Decimal) -> Self {
        let multiplier = if average_spread.is_zero() {
            Decimal::ONE
        } else {
            current_spread / average_spread
        };

        Self {
            current_spread,
            average_spread,
            spread_multiplier: multiplier,
        }
    }

    pub fn get_state(&self) -> ExecutionProtectionState {
        let m = self.spread_multiplier;
        let one_point_five = dec!(1.5);
        let two = dec!(2.0);
        let three = dec!(3.0);
        let five = dec!(5.0);

        if m <= one_point_five {
            ExecutionProtectionState::Normal
        } else if m <= two {
            ExecutionProtectionState::Warning
        } else if m <= three {
            ExecutionProtectionState::Restricted
        } else if m <= five {
            ExecutionProtectionState::Critical
        } else {
            ExecutionProtectionState::Frozen
        }
    }

    pub fn get_score(&self) -> u32 {
        let one = Decimal::ONE;
        let five = dec!(5.0);

        if self.spread_multiplier <= one {
            0
        } else if self.spread_multiplier >= five {
            100
        } else {
            let diff = self.spread_multiplier - one;
            let ratio = diff / dec!(4.0);
            let score = ratio * dec!(100.0);
            
            score.to_u32().unwrap_or(100).min(100)
        }
    }
}
