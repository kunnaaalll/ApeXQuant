use serde::{Deserialize, Serialize};

/// Represents the explicit state of a trading position.
/// No hidden transitions are permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionState {
    /// Initial state when order is placed but no fill has occurred.
    Opening,
    /// Standard state for an active, fully entered position.
    Active,
    /// State when actively adding to the position.
    ScalingIn,
    /// State when actively removing from the position (taking partial profits).
    ScalingOut,
    /// State when actively reducing the position (risk mitigation).
    Reducing,
    /// State when actively attempting to exit the entire position.
    Closing,
    /// State when position has been fully exited.
    Closed,
    /// Terminal state for historical reference.
    Archived,
    /// Error state representing invalid transitions or systemic failures.
    Invalid,
}

impl Default for PositionState {
    fn default() -> Self {
        PositionState::Opening
    }
}

impl PositionState {
    /// Validates if a transition to the next state is permitted.
    pub fn can_transition_to(&self, next: &PositionState) -> bool {
        match (self, next) {
            // Valid transitions from Opening
            (PositionState::Opening, PositionState::Active) => true,
            (PositionState::Opening, PositionState::Invalid) => true,
            (PositionState::Opening, PositionState::Closed) => true, // Order canceled

            // Valid transitions from Active
            (PositionState::Active, PositionState::ScalingIn) => true,
            (PositionState::Active, PositionState::ScalingOut) => true,
            (PositionState::Active, PositionState::Reducing) => true,
            (PositionState::Active, PositionState::Closing) => true,

            // Valid transitions from Scaling states back to Active or Closing
            (PositionState::ScalingIn, PositionState::Active) => true,
            (PositionState::ScalingIn, PositionState::Closing) => true, // Emergency close
            (PositionState::ScalingOut, PositionState::Active) => true,
            (PositionState::ScalingOut, PositionState::Closing) => true,

            // Valid transitions from Reducing
            (PositionState::Reducing, PositionState::Active) => true,
            (PositionState::Reducing, PositionState::Closing) => true,

            // Valid transitions from Closing
            (PositionState::Closing, PositionState::Closed) => true,

            // Valid transitions from Closed
            (PositionState::Closed, PositionState::Archived) => true,

            // Everything else is invalid
            _ => false,
        }
    }
}
