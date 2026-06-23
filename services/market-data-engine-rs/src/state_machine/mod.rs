#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Synchronizing,
    Healthy,
    Degraded,
    Recovering,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTransitionError {
    InvalidTransition(ConnectionState, ConnectionState),
}

impl std::fmt::Display for StateTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTransition(from, to) => write!(f, "Invalid state transition from {:?} to {:?}", from, to),
        }
    }
}

impl std::error::Error for StateTransitionError {}

impl ConnectionState {
    pub fn transition(&self, next: ConnectionState) -> Result<ConnectionState, StateTransitionError> {
        match (self, next) {
            (Self::Disconnected, Self::Connecting) => Ok(next),
            
            (Self::Connecting, Self::Synchronizing) => Ok(next),
            (Self::Connecting, Self::Failed) => Ok(next),
            (Self::Connecting, Self::Disconnected) => Ok(next),
            
            (Self::Synchronizing, Self::Healthy) => Ok(next),
            (Self::Synchronizing, Self::Failed) => Ok(next),
            (Self::Synchronizing, Self::Disconnected) => Ok(next),
            
            (Self::Healthy, Self::Degraded) => Ok(next),
            (Self::Healthy, Self::Failed) => Ok(next),
            (Self::Healthy, Self::Disconnected) => Ok(next),
            
            (Self::Degraded, Self::Recovering) => Ok(next),
            (Self::Degraded, Self::Failed) => Ok(next),
            (Self::Degraded, Self::Disconnected) => Ok(next),
            
            (Self::Recovering, Self::Healthy) => Ok(next),
            (Self::Recovering, Self::Degraded) => Ok(next),
            (Self::Recovering, Self::Failed) => Ok(next),
            (Self::Recovering, Self::Disconnected) => Ok(next),
            
            (Self::Failed, Self::Connecting) => Ok(next),
            (Self::Failed, Self::Disconnected) => Ok(next),

            (current, new_state) if current == &new_state => Ok(new_state),

            _ => Err(StateTransitionError::InvalidTransition(*self, next)),
        }
    }
}
