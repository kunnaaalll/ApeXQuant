use crate::brokers::errors::BrokerError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailoverState {
    Healthy,
    Warning,
    Failover,
    Recovery,
}

impl Default for FailoverState {
    fn default() -> Self {
        Self::Healthy
    }
}

impl FailoverState {
    pub fn transition_to(&mut self, new_state: FailoverState) -> Result<(), BrokerError> {
        let valid = match (*self, new_state) {
            (FailoverState::Healthy, FailoverState::Warning) => true,
            (FailoverState::Healthy, FailoverState::Failover) => true,
            (FailoverState::Warning, FailoverState::Failover) => true,
            (FailoverState::Warning, FailoverState::Healthy) => true,
            (FailoverState::Failover, FailoverState::Recovery) => true,
            (FailoverState::Failover, FailoverState::Healthy) => false, // Strictly forbidden by spec
            (FailoverState::Recovery, FailoverState::Warning) => true,
            (FailoverState::Recovery, FailoverState::Failover) => true,
            (current, next) if current == next => true,
            _ => false,
        };

        if valid {
            *self = new_state;
            Ok(())
        } else {
            Err(BrokerError::InvalidStateTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", new_state),
            })
        }
    }
}
