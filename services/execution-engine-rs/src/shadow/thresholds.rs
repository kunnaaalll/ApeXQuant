use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParityThresholds {
    pub exact_match: Decimal,
    pub close_match: Decimal,
    pub warning: Decimal,
    pub mismatch: Decimal,
}

impl ParityThresholds {
    pub const fn institutional() -> Self {
        Self {
            exact_match: dec!(1.0),
            close_match: dec!(3.0),
            warning: dec!(5.0),
            mismatch: dec!(10.0),
        }
    }
}

impl Default for ParityThresholds {
    fn default() -> Self {
        Self::institutional()
    }
}
