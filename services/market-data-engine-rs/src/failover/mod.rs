#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailoverState {
    Healthy,
    Warning,
    Failover,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailoverTransitionError {
    InvalidSequence(FailoverState, FailoverState),
}

impl std::fmt::Display for FailoverTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSequence(from, to) => write!(f, "Invalid failover sequence from {:?} to {:?}", from, to),
        }
    }
}

impl std::error::Error for FailoverTransitionError {}

impl FailoverState {
    pub fn transition(&self, next: FailoverState) -> Result<FailoverState, FailoverTransitionError> {
        match (self, next) {
            (Self::Healthy, Self::Warning) => Ok(next),
            (Self::Warning, Self::Failover) => Ok(next),
            (Self::Warning, Self::Healthy) => Ok(next),
            (Self::Failover, Self::Recovery) => Ok(next),
            (Self::Recovery, Self::Healthy) => Ok(next),
            (Self::Recovery, Self::Failover) => Ok(next), // e.g. fails again during recovery
            (current, new_state) if current == &new_state => Ok(new_state),
            _ => Err(FailoverTransitionError::InvalidSequence(*self, next)),
        }
    }
}
