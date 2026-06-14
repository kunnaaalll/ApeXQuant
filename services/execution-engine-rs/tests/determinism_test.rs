use proptest::prelude::*;
use execution_engine::state_machine::{StateMachine, states::OrderState, events::ExecutionEvent};

proptest! {
    #[test]
    fn state_machine_determinism(
        initial_state_idx in 0..6u32, 
        event_idx in 0..5u32
    ) {
        let initial_state = match initial_state_idx {
            0 => OrderState::Pending,
            1 => OrderState::Submitted,
            2 => OrderState::Accepted,
            3 => OrderState::PartiallyFilled,
            4 => OrderState::Filled,
            _ => OrderState::Unknown,
        };

        let event = match event_idx {
            0 => ExecutionEvent::Submit,
            1 => ExecutionEvent::BrokerAccepted,
            2 => ExecutionEvent::FillPartial { quantity: "10".to_string(), price: "1.0".to_string() },
            3 => ExecutionEvent::FillComplete { price: "1.0".to_string() },
            _ => ExecutionEvent::BrokerRejected { reason: "test".to_string() },
        };

        let result1 = StateMachine::transition(initial_state.clone(), event.clone(), 1234567890);
        let result2 = StateMachine::transition(initial_state, event, 1234567890);

        // Determinism rule: exact same initial state + exact same event MUST yield exact same output state.
        match (result1, result2) {
            (Ok(r1), Ok(r2)) => {
                assert_eq!(r1.previous_state, r2.previous_state);
                assert_eq!(r1.new_state, r2.new_state);
            },
            (Err(_), Err(_)) => {
                // Both rejected the invalid transition
            },
            _ => panic!("Non-deterministic transition outcome!"),
        }
    }
}
