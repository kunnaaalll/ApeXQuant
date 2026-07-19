use crate::streaks::*;
#[cfg(test)]
use rust_decimal_macros::dec;

#[test]
fn test_streak_detector_critical() {
    let detector = StreakDetector::evaluate(-6, 3, 4, dec!(1.5), dec!(2.0));
    assert_eq!(detector.state, StreakState::Critical);
}

#[test]
fn test_recovery_factor() {
    let recovery = RecoveryFactor::calculate(5, 10);
    assert_eq!(recovery.recovery_progress, dec!(0.5));
    assert!(recovery.in_recovery);

    let completed = RecoveryFactor::calculate(12, 10);
    assert_eq!(completed.recovery_progress, dec!(1.0));
    assert!(!completed.in_recovery);
}
