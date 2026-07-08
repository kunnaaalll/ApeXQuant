use super::state::ConnectionState;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct ConnectionSupervisor {
    pub state: ConnectionState,
    pub last_heartbeat: Option<Instant>,
    pub reconnect_attempts: u32,
    pub auth_status: bool,
}

impl ConnectionSupervisor {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            last_heartbeat: None,
            reconnect_attempts: 0,
            auth_status: false,
        }
    }

    pub fn transition(&mut self, new_state: ConnectionState) {
        self.state = new_state;
    }

    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
        if self.state == ConnectionState::Degraded || self.state == ConnectionState::Recovering {
            self.state = ConnectionState::Healthy;
        }
    }

    pub fn check_health(&mut self, timeout: Duration) -> ConnectionState {
        if let Some(last) = self.last_heartbeat {
            if last.elapsed() > timeout {
                self.state = ConnectionState::Degraded;
            }
        }
        self.state
    }

    pub fn record_auth_success(&mut self) {
        self.auth_status = true;
        self.state = ConnectionState::Healthy;
        self.reconnect_attempts = 0;
    }

    pub fn record_auth_failure(&mut self) {
        self.auth_status = false;
        self.state = ConnectionState::Failed;
    }

    pub fn attempt_reconnect(&mut self) {
        self.reconnect_attempts += 1;
        self.state = ConnectionState::Connecting;
    }
}
