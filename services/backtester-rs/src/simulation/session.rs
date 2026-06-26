use crate::market_replay::engine::ReplayEngine;
use crate::simulation::state::SimulationState;
use crate::simulation::metrics::SimulationMetrics;

pub struct SimulationSession<T: ReplayEngine> {
    pub id: String,
    pub state: SimulationState,
    pub engine: T,
    pub metrics: SimulationMetrics,
}

impl<T: ReplayEngine> SimulationSession<T> {
    pub fn new(id: String, engine: T) -> Self {
        Self {
            id,
            state: SimulationState::Created,
            engine,
            metrics: SimulationMetrics::default(),
        }
    }

    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.state.can_transition_to(SimulationState::Loading) {
            self.state = SimulationState::Loading;
            // Load resources...
            self.state = SimulationState::Running;
            Ok(())
        } else {
            Err("Invalid state transition to start")
        }
    }

    pub fn pause(&mut self) -> Result<(), &'static str> {
        if self.state.can_transition_to(SimulationState::Paused) {
            self.state = SimulationState::Paused;
            self.engine.pause();
            Ok(())
        } else {
            Err("Invalid state transition to pause")
        }
    }

    pub fn resume(&mut self) -> Result<(), &'static str> {
        if self.state.can_transition_to(SimulationState::Running) {
            self.state = SimulationState::Running;
            self.engine.resume();
            Ok(())
        } else {
            Err("Invalid state transition to resume")
        }
    }

    pub fn tick(&mut self) -> Result<bool, &'static str> {
        if self.state != SimulationState::Running {
            return Ok(false);
        }

        match self.engine.next_event()? {
            Some(event) => {
                self.metrics.events_processed += 1;
                // Dispatch event to bus
                let _ts = event.timestamp();
                Ok(true)
            }
            None => {
                self.state = SimulationState::Completed;
                Ok(false)
            }
        }
    }
}
