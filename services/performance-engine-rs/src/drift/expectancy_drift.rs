use crate::drift::DriftState;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct ExpectancyDrift {
    pub difference: Decimal,
    pub percentage_change: Decimal,
    pub acceleration: Decimal,
    pub trend_direction: i8,
    pub state: DriftState,
}

impl ExpectancyDrift {
    pub fn evaluate(
        current_expectancy: Decimal,
        previous_expectancy: Decimal,
        historical_avg: Decimal,
    ) -> Self {
        let difference = current_expectancy - historical_avg;

        let percentage_change = if historical_avg.abs() > Decimal::ZERO {
            difference / historical_avg.abs()
        } else {
            Decimal::ZERO
        };

        let velocity = current_expectancy - previous_expectancy;
        let acceleration = velocity;

        let trend_direction = if velocity > dec!(0.02) {
            1
        } else if velocity < -dec!(0.02) {
            -1
        } else {
            0
        };

        let state = if percentage_change < -dec!(0.2) {
            DriftState::Critical
        } else if percentage_change < -dec!(0.05) {
            DriftState::Weakening
        } else if percentage_change > dec!(0.05) {
            DriftState::Improving
        } else {
            DriftState::Stable
        };

        Self {
            difference,
            percentage_change,
            acceleration,
            trend_direction,
            state,
        }
    }
}
