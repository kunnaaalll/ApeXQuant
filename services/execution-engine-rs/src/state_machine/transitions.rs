use serde::{Deserialize, Serialize};
use super::{events::ExecutionEvent, states::OrderState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransitionResult {
    pub previous_state: OrderState,
    pub new_state: OrderState,
    pub event: ExecutionEvent,
    pub timestamp: i64,
    pub persisted: bool,
}

pub struct StateMachine;

impl StateMachine {
    /// Deterministically transitions from one state to another based on an event.
    /// Ensures illegal transitions are impossible and provides a result that must be persisted.
    pub fn transition(current: OrderState, event: ExecutionEvent, timestamp: i64) -> Result<StateTransitionResult, TransitionError> {
        let new_state = match (&current, &event) {
            (OrderState::Pending, ExecutionEvent::Submit) => Ok(OrderState::Submitted),
            (OrderState::Pending, ExecutionEvent::Cancelled) => Ok(OrderState::Cancelled),
            
            (OrderState::Submitted, ExecutionEvent::BrokerAccepted) => Ok(OrderState::Accepted),
            (OrderState::Submitted, ExecutionEvent::BrokerRejected { .. }) => Ok(OrderState::Rejected),
            (OrderState::Submitted, ExecutionEvent::DesyncDetected) => Ok(OrderState::Unknown),
            
            (OrderState::Accepted, ExecutionEvent::FillPartial { .. }) => Ok(OrderState::PartiallyFilled),
            (OrderState::Accepted, ExecutionEvent::FillComplete { .. }) => Ok(OrderState::Filled),
            (OrderState::Accepted, ExecutionEvent::Cancelled) => Ok(OrderState::Cancelled),
            (OrderState::Accepted, ExecutionEvent::Expired) => Ok(OrderState::Expired),
            
            (OrderState::PartiallyFilled, ExecutionEvent::FillPartial { .. }) => Ok(OrderState::PartiallyFilled),
            (OrderState::PartiallyFilled, ExecutionEvent::FillComplete { .. }) => Ok(OrderState::Filled),
            (OrderState::PartiallyFilled, ExecutionEvent::Cancelled) => Ok(OrderState::Closed), // Remaining size cancelled
            
            (OrderState::Filled, ExecutionEvent::Manage) => Ok(OrderState::Managed),
            (OrderState::Filled, ExecutionEvent::PositionClosed) => Ok(OrderState::Closed),
            
            (OrderState::Managed, ExecutionEvent::PositionClosed) => Ok(OrderState::Closed),
            
            (OrderState::Closed, ExecutionEvent::Archive) => Ok(OrderState::Archived),
            (OrderState::Rejected, ExecutionEvent::Archive) => Ok(OrderState::Archived),
            (OrderState::Cancelled, ExecutionEvent::Archive) => Ok(OrderState::Archived),
            (OrderState::Expired, ExecutionEvent::Archive) => Ok(OrderState::Archived),
            
            // Reconciliation and Recovery
            (_, ExecutionEvent::DesyncDetected) => Ok(OrderState::Unknown),
            (OrderState::Unknown, ExecutionEvent::RecoveryStarted) => Ok(OrderState::Recovering),
            (OrderState::Recovering, ExecutionEvent::RecoveryCompleted) => Ok(OrderState::Managed),
            
            (state, evt) => Err(TransitionError::InvalidTransition { 
                state: *state, 
                event: evt.clone() 
            }),
        }?;

        Ok(StateTransitionResult {
            previous_state: current,
            new_state,
            event,
            timestamp,
            persisted: false, // Must be marked true by the storage layer
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("Invalid transition: cannot process {event:?} from {state:?}")]
    InvalidTransition {
        state: OrderState,
        event: ExecutionEvent,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_transitions() {
        let ts = 1700000000;
        let res = StateMachine::transition(OrderState::Pending, ExecutionEvent::Submit, ts).unwrap();
        assert_eq!(res.new_state, OrderState::Submitted);
        assert_eq!(res.previous_state, OrderState::Pending);
        assert_eq!(res.persisted, false);
        
        let res2 = StateMachine::transition(OrderState::Submitted, ExecutionEvent::BrokerAccepted, ts).unwrap();
        assert_eq!(res2.new_state, OrderState::Accepted);
    }
    
    #[test]
    fn test_invalid_transitions() {
        let ts = 1700000000;
        assert!(StateMachine::transition(OrderState::Pending, ExecutionEvent::BrokerAccepted, ts).is_err());
    }
}
