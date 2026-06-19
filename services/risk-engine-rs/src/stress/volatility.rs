use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolatilityState {
    Normal,
    Elevated,
    High,
    Extreme,
    Collapse,
}

pub struct VolatilityShockEngine {
    base_volatility: Decimal,
}

impl VolatilityShockEngine {
    pub fn new(base_volatility: Decimal) -> Self {
        Self {
            base_volatility: base_volatility.max(dec!(0)),
        }
    }

    pub fn apply_shock(&self, multiplier: Decimal) -> Decimal {
        let mut shocked = self.base_volatility * multiplier;
        if shocked < dec!(0) {
            shocked = dec!(0);
        }
        shocked
    }

    pub fn evaluate_state(&self, shocked_volatility: Decimal) -> VolatilityState {
        if shocked_volatility < dec!(0.15) {
            VolatilityState::Normal
        } else if shocked_volatility < dec!(0.30) {
            VolatilityState::Elevated
        } else if shocked_volatility < dec!(0.50) {
            VolatilityState::High
        } else if shocked_volatility < dec!(0.80) {
            VolatilityState::Extreme
        } else {
            VolatilityState::Collapse
        }
    }
}
