#![allow(warnings, clippy::all, deprecated)]
use crate::streaks::{RecoveryFactor, StreakDetector, StreakImpact};
use rust_decimal::Decimal;

#[test]
fn test_streak_detector() {
    let mut detector = StreakDetector::new();
    assert_eq!(detector.impact(), StreakImpact::Neutral);

    detector.record_win();
    detector.record_win();
    detector.record_win();
    assert_eq!(detector.impact(), StreakImpact::Positive);

    detector.record_loss();
    assert_eq!(detector.impact(), StreakImpact::Neutral);

    detector.record_loss();
    detector.record_loss();
    assert_eq!(detector.impact(), StreakImpact::Negative);

    detector.record_loss();
    detector.record_loss();
    assert_eq!(detector.impact(), StreakImpact::Critical);
}

#[test]
fn test_recovery_factor() {
    let factor1 = RecoveryFactor::calculate(1);
    assert_eq!(factor1.amount, Decimal::new(1, 1)); // 0.1

    let factor2 = RecoveryFactor::calculate(5);
    assert_eq!(factor2.amount, Decimal::new(5, 1)); // 0.5

    let factor3 = RecoveryFactor::calculate(20);
    // clamped to 1.0
    assert_eq!(factor3.amount, Decimal::new(10, 1));
}
