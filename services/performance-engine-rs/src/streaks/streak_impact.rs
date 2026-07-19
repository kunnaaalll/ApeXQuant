use crate::streaks::streak_detector::{StreakDetector, StreakState};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct StreakImpact;

impl StreakImpact {
    pub fn calculate_multiplier(detector: &StreakDetector) -> Decimal {
        match detector.state {
            StreakState::Positive => dec!(1.1),
            StreakState::Neutral => dec!(1.0),
            StreakState::Negative => dec!(0.8),
            StreakState::Critical => dec!(0.5),
        }
    }
}
