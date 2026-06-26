use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemGovernanceState {
    #[default]
    Normal,
    EmergencyFreeze,
    TradingDisabled,
    ShadowOnly,
    ReplayMode,
    MaintenanceMode,
}

pub struct GovernanceController {
    current_state: SystemGovernanceState,
}

impl Default for GovernanceController {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernanceController {
    pub fn new() -> Self {
        Self {
            current_state: SystemGovernanceState::default(),
        }
    }

    pub fn get_state(&self) -> SystemGovernanceState {
        self.current_state
    }

    pub fn set_state(&mut self, state: SystemGovernanceState) -> Result<(), &'static str> {
        self.current_state = state;
        Ok(())
    }

    pub fn trigger_emergency_freeze(&mut self) -> Result<(), &'static str> {
        self.set_state(SystemGovernanceState::EmergencyFreeze)
    }

    pub fn disable_trading(&mut self) -> Result<(), &'static str> {
        self.set_state(SystemGovernanceState::TradingDisabled)
    }

    pub fn enable_shadow_mode(&mut self) -> Result<(), &'static str> {
        self.set_state(SystemGovernanceState::ShadowOnly)
    }
}
