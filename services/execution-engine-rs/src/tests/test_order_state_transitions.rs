use crate::state::ExecutionState;

#[test]
fn test_order_state_transitions() {
    let mut state = ExecutionState::Idle;

    assert!(state.transition_to(ExecutionState::Submitting).is_ok());
    assert_eq!(state, ExecutionState::Submitting);

    assert!(state.transition_to(ExecutionState::Waiting).is_ok());
    assert_eq!(state, ExecutionState::Waiting);

    assert!(state.transition_to(ExecutionState::Filled).is_ok());
    assert_eq!(state, ExecutionState::Filled);

    // Forbidden transition
    let err = state.transition_to(ExecutionState::Submitting);
    assert!(err.is_err());
    assert_eq!(state, ExecutionState::Filled);

    let mut state2 = ExecutionState::Failed;
    // Forbidden transition
    let err2 = state2.transition_to(ExecutionState::Filled);
    assert!(err2.is_err());
    assert_eq!(state2, ExecutionState::Failed);
}
