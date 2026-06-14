use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    pub actual_outcome: Decimal,
    pub alternate_outcome: Decimal,
    pub difference: Decimal,
    pub confidence: Decimal,
    pub reason: String,
}

impl CounterfactualResult {
    pub fn new(
        actual_outcome: Decimal,
        alternate_outcome: Decimal,
        confidence: Decimal,
        reason: String,
    ) -> Self {
        Self {
            actual_outcome,
            alternate_outcome,
            difference: alternate_outcome - actual_outcome,
            confidence,
            reason,
        }
    }
}
