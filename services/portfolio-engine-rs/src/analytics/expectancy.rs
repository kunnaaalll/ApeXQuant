// src/analytics/expectancy.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpectancyState {
    Exceptional,
    Strong,
    Normal,
    Weak,
    Negative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpectancyAssessment {
    pub expectancy: Decimal,
    pub expectancy_per_trade: Decimal,
    pub expectancy_per_day: Decimal,
    pub expectancy_per_week: Decimal,
    pub rolling_expectancy: Decimal,
    pub state: ExpectancyState,
}

impl ExpectancyAssessment {
    /// Evaluate the expectancy state based on the calculated expectancy value.
    /// Thresholds can be adjusted, but APEX expects these to reflect institutional
    /// standard edge sizing.
    pub fn evaluate_state(expectancy: Decimal) -> ExpectancyState {
        if expectancy < Decimal::ZERO {
            ExpectancyState::Negative
        } else if expectancy < Decimal::new(1, 1) {
            ExpectancyState::Weak
        } else if expectancy < Decimal::new(25, 2) {
            ExpectancyState::Normal
        } else if expectancy < Decimal::new(5, 1) {
            ExpectancyState::Strong
        } else {
            ExpectancyState::Exceptional
        }
    }

    /// Factory for creating an assessment enforcing all invariants
    pub fn new(
        expectancy: Decimal,
        expectancy_per_trade: Decimal,
        expectancy_per_day: Decimal,
        expectancy_per_week: Decimal,
        rolling_expectancy: Decimal,
    ) -> Self {
        let state = Self::evaluate_state(expectancy);

        Self {
            expectancy,
            expectancy_per_trade,
            expectancy_per_day,
            expectancy_per_week,
            rolling_expectancy,
            state,
        }
    }
}
