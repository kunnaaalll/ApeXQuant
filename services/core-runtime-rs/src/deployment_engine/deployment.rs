#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentMode {
    Rolling,
    BlueGreen,
    Canary,
    Shadow,
    EmergencyRollback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentState {
    Pending,
    InProgress,
    VerifyingHealth,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone)]
pub struct DeploymentEngine {
    pub current_mode: DeploymentMode,
    pub current_state: DeploymentState,
}

impl DeploymentEngine {
    pub fn new(mode: DeploymentMode) -> Self {
        Self {
            current_mode: mode,
            current_state: DeploymentState::Pending,
        }
    }

    pub fn start_deployment(&mut self) {
        self.current_state = DeploymentState::InProgress;
    }

    pub fn verify_health(&mut self) {
        if self.current_state == DeploymentState::InProgress {
            self.current_state = DeploymentState::VerifyingHealth;
        }
    }

    pub fn complete_deployment(&mut self) {
        if self.current_state == DeploymentState::VerifyingHealth {
            self.current_state = DeploymentState::Completed;
        }
    }

    pub fn trigger_rollback(&mut self) {
        self.current_mode = DeploymentMode::EmergencyRollback;
        self.current_state = DeploymentState::RolledBack;
    }
}

impl Default for DeploymentEngine {
    fn default() -> Self {
        Self::new(DeploymentMode::Rolling)
    }
}
