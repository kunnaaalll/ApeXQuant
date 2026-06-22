use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlippageState {
    Healthy,
    Elevated,
    Danger,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlippageGuards {
    pub expected_slippage: Decimal,
    pub realized_slippage: Decimal,
}

impl SlippageGuards {
    pub fn new(expected_slippage: Decimal, realized_slippage: Decimal) -> Self {
        let zero = Decimal::ZERO;
        Self {
            expected_slippage: expected_slippage.max(zero),
            realized_slippage: realized_slippage.max(zero),
        }
    }

    pub fn slippage_drift(&self) -> Decimal {
        if self.expected_slippage.is_zero() {
            self.realized_slippage
        } else {
            let drift = self.realized_slippage - self.expected_slippage;
            drift.max(Decimal::ZERO)
        }
    }

    pub fn get_state(&self) -> SlippageState {
        let _drift = self.slippage_drift();
        // Assuming drift in absolute price or pips. We will use a multiplier for state if we want,
        // but absolute drift makes sense. Or ratio:
        let ratio = if self.expected_slippage.is_zero() {
            if self.realized_slippage > dec!(0.0001) { dec!(10.0) } else { Decimal::ONE }
        } else {
            self.realized_slippage / self.expected_slippage
        };

        if ratio <= dec!(1.5) {
            SlippageState::Healthy
        } else if ratio <= dec!(2.5) {
            SlippageState::Elevated
        } else if ratio <= dec!(4.0) {
            SlippageState::Danger
        } else {
            SlippageState::Collapse
        }
    }

    pub fn get_penalty_score(&self) -> u32 {
        let ratio = if self.expected_slippage.is_zero() {
            if self.realized_slippage.is_zero() {
                Decimal::ONE
            } else {
                dec!(5.0) // Arbitrary high ratio for unexpected slippage
            }
        } else {
            self.realized_slippage / self.expected_slippage
        };

        if ratio <= Decimal::ONE {
            0
        } else if ratio >= dec!(5.0) {
            100
        } else {
            let diff = ratio - Decimal::ONE;
            let percent = diff / dec!(4.0) * dec!(100.0);
            percent.to_u32().unwrap_or(100).min(100)
        }
    }
}
