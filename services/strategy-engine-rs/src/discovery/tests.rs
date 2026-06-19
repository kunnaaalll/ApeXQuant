use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_edge_discovery() {
    let mut discovery = EdgeDiscovery::new();

    // Strengthening: > 0.10
    let state = discovery.detect(dec!(0.05), dec!(0.04), dec!(0.02));
    assert_eq!(state, EdgeState::Strengthening);

    // Emerging: > 0.03
    let state = discovery.detect(dec!(0.02), dec!(0.01), dec!(0.01));
    assert_eq!(state, EdgeState::Emerging);

    // Collapsing: <= -0.10
    let state = discovery.detect(dec!(-0.05), dec!(-0.05), dec!(-0.01));
    assert_eq!(state, EdgeState::Collapsing);

    // Weakening: < -0.03
    let state = discovery.detect(dec!(-0.02), dec!(-0.01), dec!(-0.01));
    assert_eq!(state, EdgeState::Weakening);

    // Stable
    let state = discovery.detect(dec!(0.01), dec!(0.01), dec!(0.00));
    assert_eq!(state, EdgeState::Stable);
}

#[test]
fn test_velocity_detection() {
    let mut velocity = VelocityEngine::new();

    // Accelerating
    let state = velocity.update(dec!(0.05));
    assert_eq!(state, VelocityState::Accelerating); // previous = 0.0, current = 0.05 => accel > 0.01

    // Stable
    let state = velocity.update(dec!(0.055));
    assert_eq!(state, VelocityState::Stable); // previous = 0.05, current = 0.055 => accel < 0.01 && > -0.01

    // Decelerating
    let state = velocity.update(dec!(0.03));
    assert_eq!(state, VelocityState::Decelerating); // previous = 0.055, current = 0.03 => accel = -0.025 < -0.01

    // Reversing
    let state = velocity.update(dec!(-0.01));
    assert_eq!(state, VelocityState::Reversing); // crossed zero
}

#[test]
fn test_collapse_detection() {
    let mut detector = DeteriorationDetector::new();

    // Healthy -> Critical (Immediate downgrade)
    let state = detector.update(dec!(0.10), dec!(0.10), dec!(0.05), dec!(0.05)); // Total 0.30 => Critical
    assert_eq!(state, DeteriorationState::Critical);

    // Critical -> Collapse (Immediate downgrade)
    let state = detector.update(dec!(0.20), dec!(0.10), dec!(0.10), dec!(0.05)); // Total 0.45 => Collapse
    assert_eq!(state, DeteriorationState::Collapse);

    // Collapse -> Critical (Gradual recovery)
    let state = detector.update(dec!(0.0), dec!(0.0), dec!(0.0), dec!(0.0)); // Total 0.00 => Target Healthy
    assert_eq!(state, DeteriorationState::Critical); // Gradual step

    // Critical -> Danger (Gradual recovery)
    let state = detector.update(dec!(0.0), dec!(0.0), dec!(0.0), dec!(0.0));
    assert_eq!(state, DeteriorationState::Danger);
}
