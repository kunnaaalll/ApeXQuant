#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    Created,
    Loading,
    Running,
    Paused,
    Completed,
    Failed,
}

impl SimulationState {
    pub fn can_transition_to(&self, new_state: SimulationState) -> bool {
        matches!(
            (self, new_state),
            (SimulationState::Created, SimulationState::Loading)
                | (SimulationState::Loading, SimulationState::Running)
                | (SimulationState::Loading, SimulationState::Failed)
                | (SimulationState::Running, SimulationState::Paused)
                | (SimulationState::Running, SimulationState::Completed)
                | (SimulationState::Running, SimulationState::Failed)
                | (SimulationState::Paused, SimulationState::Running)
                | (SimulationState::Paused, SimulationState::Failed)
        )
    }
}
