use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftState {
    None,
    Low,
    Moderate,
    High,
    Extreme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftAnalysis {
    pub absolute_drift: Decimal,
    pub relative_drift: Decimal,
    pub state: DriftState,
}

pub struct DriftEngine;

impl DriftEngine {
    pub fn calculate(expected: Decimal, actual: Decimal) -> DriftAnalysis {
        let absolute_drift = (expected - actual).abs();

        let relative_drift = if expected.is_zero() {
            if actual.is_zero() {
                dec!(0)
            } else {
                dec!(100)
            }
        } else {
            let diff_percent = (absolute_drift / expected.abs()) * dec!(100);
            if diff_percent > dec!(100) {
                dec!(100)
            } else {
                diff_percent
            }
        };

        let state = if relative_drift.is_zero() {
            DriftState::None
        } else if relative_drift <= dec!(1.0) {
            DriftState::Low
        } else if relative_drift <= dec!(5.0) {
            DriftState::Moderate
        } else if relative_drift <= dec!(10.0) {
            DriftState::High
        } else {
            DriftState::Extreme
        };

        DriftAnalysis {
            absolute_drift,
            relative_drift,
            state,
        }
    }
}
