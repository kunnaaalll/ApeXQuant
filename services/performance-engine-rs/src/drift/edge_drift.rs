use crate::drift::DriftState;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct EdgeDrift {
    pub difference: Decimal,
    pub percentage_change: Decimal,
    pub acceleration: Decimal,
    pub trend_direction: i8,
    pub state: DriftState,
}

impl EdgeDrift {
    pub fn evaluate(
        current_edge: Decimal,
        previous_edge: Decimal,
        historical_avg: Decimal,
    ) -> Self {
        let difference = current_edge - historical_avg;

        let percentage_change = if historical_avg.abs() > Decimal::ZERO {
            difference / historical_avg.abs()
        } else {
            Decimal::ZERO
        };

        let current_velocity = current_edge - previous_edge;
        let acceleration = current_velocity;

        let trend_direction = if current_velocity > dec!(0.01) {
            1
        } else if current_velocity < -dec!(0.01) {
            -1
        } else {
            0
        };

        let state = if percentage_change < -dec!(0.2) || acceleration < -dec!(0.1) {
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
