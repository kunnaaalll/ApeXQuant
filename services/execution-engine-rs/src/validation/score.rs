use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationLevel {
    Broken,
    Weak,
    Normal,
    Strong,
    Elite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationScore {
    pub value: Decimal,
}

impl Default for ValidationScore {
    fn default() -> Self {
        Self { value: dec!(0) }
    }
}

impl ValidationScore {
    pub fn new(value: Decimal) -> Self {
        let mut v = value;
        if v < dec!(0) {
            v = dec!(0);
        } else if v > dec!(100) {
            v = dec!(100);
        }
        Self { value: v }
    }

    pub fn level(&self) -> ValidationLevel {
        if self.value >= dec!(90) {
            ValidationLevel::Elite
        } else if self.value >= dec!(75) {
            ValidationLevel::Strong
        } else if self.value >= dec!(50) {
            ValidationLevel::Normal
        } else if self.value >= dec!(25) {
            ValidationLevel::Weak
        } else {
            ValidationLevel::Broken
        }
    }
}
