use super::circuit_breaker::ExecutionProtectionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureState {
    Stable,
    Warning,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureTracker {
    pub broker_errors: u32,
    pub timeouts: u32,
    pub routing_failures: u32,
    pub connection_drops: u32,
}

impl FailureTracker {
    pub fn new(broker_errors: u32, timeouts: u32, routing_failures: u32, connection_drops: u32) -> Self {
        Self {
            broker_errors,
            timeouts,
            routing_failures,
            connection_drops,
        }
    }

    pub fn get_score(&self) -> u32 {
        let mut score = 0;
        
        // Weights
        score += self.broker_errors.saturating_mul(10);
        score += self.timeouts.saturating_mul(5);
        score += self.routing_failures.saturating_mul(15);
        score += self.connection_drops.saturating_mul(25);
        
        score.min(100)
    }

    pub fn get_failure_state(&self) -> FailureState {
        let score = self.get_score();
        if score >= 80 {
            FailureState::Collapse
        } else if score >= 50 {
            FailureState::Critical
        } else if score >= 20 {
            FailureState::Warning
        } else {
            FailureState::Stable
        }
    }

    pub fn get_protection_state(&self) -> ExecutionProtectionState {
        match self.get_failure_state() {
            FailureState::Stable => ExecutionProtectionState::Normal,
            FailureState::Warning => ExecutionProtectionState::Restricted,
            FailureState::Critical => ExecutionProtectionState::Critical,
            FailureState::Collapse => ExecutionProtectionState::Frozen,
        }
    }
}
