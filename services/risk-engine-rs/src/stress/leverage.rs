use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeverageState {
    Stable,
    Elevated,
    Danger,
    Collapse,
}

pub struct LeverageCascadeEngine {
    gross_leverage: Decimal,
}

impl LeverageCascadeEngine {
    pub fn new(gross_leverage: Decimal) -> Self {
        Self {
            gross_leverage: gross_leverage.max(dec!(1.0)),
        }
    }

    pub fn apply_cascade(&self, amplification: Decimal) -> Decimal {
        let mut new_leverage = self.gross_leverage * amplification;
        if new_leverage < dec!(1.0) {
            new_leverage = dec!(1.0);
        }
        new_leverage
    }

    pub fn evaluate_state(&self, stressed_leverage: Decimal) -> LeverageState {
        if stressed_leverage < dec!(2.0) {
            LeverageState::Stable
        } else if stressed_leverage < dec!(4.0) {
            LeverageState::Elevated
        } else if stressed_leverage < dec!(8.0) {
            LeverageState::Danger
        } else {
            LeverageState::Collapse
        }
    }
}
