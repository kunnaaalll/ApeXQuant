use super::strategy_degradation::DegradationState;

#[test]
fn test_collapse_thresholds() {
    let state = DegradationState::Collapse;
    assert_eq!(state, DegradationState::Collapse);
}
