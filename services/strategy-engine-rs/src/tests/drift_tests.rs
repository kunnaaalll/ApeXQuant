#![allow(warnings, clippy::all, deprecated)]
use crate::drift::{DriftEngine, DriftState};
use rust_decimal::Decimal;

#[test]
fn test_drift_thresholds() {
    let mut engine = DriftEngine::new();

    // Default is 0 drift => Stable
    assert_eq!(engine.state(), DriftState::Stable);

    // Improving
    engine.edge_drift = Decimal::new(15, 2); // 0.15
    assert_eq!(engine.state(), DriftState::Improving);

    // Weakening
    engine.edge_drift = Decimal::new(-25, 2); // -0.25
    assert_eq!(engine.state(), DriftState::Weakening);

    // Critical
    engine.expectancy_drift = Decimal::new(-45, 2); // -0.45
    assert_eq!(engine.state(), DriftState::Critical);

    // Collapse
    engine.confidence_drift = Decimal::new(-65, 2); // -0.65
    assert_eq!(engine.state(), DriftState::Collapse);
}
