use crate::brokers::errors::BrokerError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Degraded,
    Reconnecting,
    Failed,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl ConnectionState {
    pub fn transition_to(&mut self, new_state: ConnectionState) -> Result<(), BrokerError> {
        let valid = match (*self, new_state) {
            (ConnectionState::Disconnected, ConnectionState::Connecting) => true,
            (ConnectionState::Connecting, ConnectionState::Connected) => true,
            (ConnectionState::Connecting, ConnectionState::Failed) => true,
            (ConnectionState::Connected, ConnectionState::Degraded) => true,
            (ConnectionState::Connected, ConnectionState::Disconnected) => true,
            (ConnectionState::Degraded, ConnectionState::Connected) => true,
            (ConnectionState::Degraded, ConnectionState::Reconnecting) => true,
            (ConnectionState::Degraded, ConnectionState::Disconnected) => true,
            (ConnectionState::Reconnecting, ConnectionState::Connected) => true,
            (ConnectionState::Reconnecting, ConnectionState::Failed) => true,
            (ConnectionState::Failed, ConnectionState::Disconnected) => true,
            (ConnectionState::Failed, ConnectionState::Connecting) => true,
            (current, next) if current == next => true, // Self-transition allowed or ignored safely
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
