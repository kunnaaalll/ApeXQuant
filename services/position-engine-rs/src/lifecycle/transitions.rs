use super::events::PositionEvent;
use crate::positions::{PositionState, PositionTracker};

pub struct TransitionEngine;

impl TransitionEngine {
    /// Attempts to apply an event to a position, returning a new state if successful.
    /// Errors if the transition is invalid.
    pub fn apply_event(tracker: &mut PositionTracker, event: PositionEvent) -> Result<(), String> {
        let next_state = match (&tracker.state, event) {
            (PositionState::Opening, PositionEvent::OrderFilled { .. }) => PositionState::Active,

            (PositionState::Active, PositionEvent::ScaleInRequested { .. }) => {
                PositionState::ScalingIn
            }
            (PositionState::Active, PositionEvent::ScaleOutRequested { .. }) => {
                PositionState::ScalingOut
            }
            (PositionState::Active, PositionEvent::CloseRequested { .. }) => PositionState::Closing,

            // Allow price updates to not change the macro state
            (_, PositionEvent::MarketTick { current_price, .. }) => {
                tracker.update_price(current_price);
                return Ok(());
            }

            (PositionState::Closing, PositionEvent::PositionClosed { .. }) => PositionState::Closed,

            // Any other combination is currently unhandled or invalid
            (state, ev) => return Err(format!("Invalid transition from {:?} via {:?}", state, ev)),
        };

        if tracker.state.can_transition_to(&next_state) {
            tracker.state = next_state;
            Ok(())
        } else {
            Err(format!(
                "Illegal state transition rule from {:?} to {:?}",
                tracker.state, next_state
            ))
        }
    }
}
