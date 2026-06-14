// src/analytics/expectancy.rs
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
    pub expectancy: f64,
    pub expectancy_per_trade: f64,
    pub expectancy_per_day: f64,
    pub expectancy_per_week: f64,
    pub rolling_expectancy: f64,
    pub state: ExpectancyState,
}

impl ExpectancyAssessment {
    /// Evaluate the expectancy state based on the calculated expectancy value.
    /// Thresholds can be adjusted, but APEX expects these to reflect institutional
    /// standard edge sizing.
    pub fn evaluate_state(expectancy: f64) -> ExpectancyState {
        if expectancy < 0.0 {
            ExpectancyState::Negative
        } else if expectancy < 0.1 {
            ExpectancyState::Weak
        } else if expectancy < 0.25 {
            ExpectancyState::Normal
        } else if expectancy < 0.5 {
            ExpectancyState::Strong
        } else {
            ExpectancyState::Exceptional
        }
    }

    /// Factory for creating an assessment enforcing all invariants
    pub fn new(
        expectancy: f64,
        expectancy_per_trade: f64,
        expectancy_per_day: f64,
        expectancy_per_week: f64,
        rolling_expectancy: f64,
    ) -> Self {
        // Guard against NaN
        let safe_exp = if expectancy.is_nan() { 0.0 } else { expectancy };
        let state = Self::evaluate_state(safe_exp);

        Self {
            expectancy: safe_exp,
            expectancy_per_trade: if expectancy_per_trade.is_nan() { 0.0 } else { expectancy_per_trade },
            expectancy_per_day: if expectancy_per_day.is_nan() { 0.0 } else { expectancy_per_day },
            expectancy_per_week: if expectancy_per_week.is_nan() { 0.0 } else { expectancy_per_week },
            rolling_expectancy: if rolling_expectancy.is_nan() { 0.0 } else { rolling_expectancy },
            state,
        }
    }
}
