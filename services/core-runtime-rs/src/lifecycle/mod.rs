use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemLifecycleState {
    #[default]
    Booting,
    Running,
    Shadow,
    Maintenance,
    Emergency,
    Recovery,
    Shutdown,
}

pub struct LifecycleManager {
    current_state: SystemLifecycleState,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            current_state: SystemLifecycleState::default(),
        }
    }

    pub fn get_state(&self) -> SystemLifecycleState {
        self.current_state
    }

    pub fn transition_to(&mut self, state: SystemLifecycleState) -> Result<(), &'static str> {
        // Here we could enforce valid state transitions
        self.current_state = state;
        Ok(())
    }
}
