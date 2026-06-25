//! Simulation Module
//!
//! Tick, candle, order lifecycle, and fill simulation.

pub enum SimulationState {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
}

pub struct SimulationEngine {
    pub state: SimulationState,
}
