use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterfactualResult {
    pub actual: Decimal,
    pub alternate: Decimal,
    pub difference: Decimal,
}

impl CounterfactualResult {
    pub fn new(actual: Decimal, alternate: Decimal) -> Self {
        Self {
            actual,
            alternate,
            difference: alternate - actual,
        }
    }
}
