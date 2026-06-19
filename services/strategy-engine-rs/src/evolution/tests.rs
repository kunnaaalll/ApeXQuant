use super::regime_evolution::EvolutionState;

#[test]
fn test_strengthening_detection() {
    let state = EvolutionState::Strengthening;
    assert_eq!(state, EvolutionState::Strengthening);
}

#[test]
fn test_abandonment_detection() {
    let state = EvolutionState::Abandoned;
    assert_eq!(state, EvolutionState::Abandoned);
}
