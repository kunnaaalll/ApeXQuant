use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    Healthy,
    Warning,
    Degraded,
    Critical,
    Recovering,
    Failed,
}

#[derive(Debug, Clone)]
pub struct RuntimeSupervisor {
    pub current_state: RuntimeState,
}

impl RuntimeSupervisor {
    pub fn new() -> Self {
        Self {
            current_state: RuntimeState::Healthy,
        }
    }

    pub fn transition(&mut self, new_state: RuntimeState) {
        self.current_state = new_state;
    }
}

impl Default for RuntimeSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ServiceSupervisor {
    pub service_states: HashMap<String, RuntimeState>,
}

impl ServiceSupervisor {
    pub fn new() -> Self {
        Self {
            service_states: HashMap::new(),
        }
    }

    pub fn register_service(&mut self, service_id: String) {
        self.service_states
            .insert(service_id, RuntimeState::Healthy);
    }

    pub fn update_state(&mut self, service_id: &str, state: RuntimeState) {
        if let Some(s) = self.service_states.get_mut(service_id) {
            *s = state;
        }
    }
}

impl Default for ServiceSupervisor {
    fn default() -> Self {
        Self::new()
    }
}
