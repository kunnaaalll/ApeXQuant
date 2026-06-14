use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectancyState {
    Exceptional,
    Strong,
    Normal,
    Weak,
    Negative,
}

#[derive(Debug, Clone)]
pub struct ExpectancyAssessment {
    pub historical_expectancy: Decimal,
    pub rolling_expectancy: Decimal,
    pub recent_expectancy: Decimal,
    pub expectancy_delta: Decimal,
    pub state: ExpectancyState,
    pub positive_drift: bool,
    pub negative_drift: bool,
    pub stagnation: bool,
}

impl ExpectancyAssessment {
    pub fn evaluate(
        historical_expectancy: Decimal,
        rolling_expectancy: Decimal,
        recent_expectancy: Decimal,
    ) -> Self {
        let expectancy_delta = recent_expectancy - historical_expectancy;
        
        let margin = dec!(0.02);
        let positive_drift = expectancy_delta > margin;
        let negative_drift = expectancy_delta < -margin;
        let stagnation = expectancy_delta.abs() <= margin;

        let state = if recent_expectancy < Decimal::ZERO {
            ExpectancyState::Negative
        } else if recent_expectancy > dec!(0.5) && positive_drift {
            ExpectancyState::Exceptional
        } else if recent_expectancy > dec!(0.2) {
            ExpectancyState::Strong
        } else if recent_expectancy > dec!(0.05) {
            ExpectancyState::Normal
        } else {
            ExpectancyState::Weak
        };

        Self {
            historical_expectancy,
            rolling_expectancy,
            recent_expectancy,
            expectancy_delta,
            state,
            positive_drift,
            negative_drift,
            stagnation,
        }
    }
}
