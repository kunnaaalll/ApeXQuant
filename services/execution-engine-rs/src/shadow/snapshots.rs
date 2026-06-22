use super::events::ShadowEvent;
use super::state::ShadowState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowSnapshot {
    pub events: Vec<ShadowEvent>,
}

impl Default for ShadowSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowSnapshot {
    pub const fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn apply_event(&mut self, event: ShadowEvent) {
        self.events.push(event);
    }

    pub fn rebuild_state(&self) -> ShadowState {
        let mut state = ShadowState::new();

        for event in &self.events {
            match event {
                ShadowEvent::ComparisonRecorded(_) => {
                    // No direct mutation to state purely from ComparisonRecorded here, 
                    // it affects statistics in the overarching logic before being emitted.
                }
                ShadowEvent::DriftCalculated(drift) => {
                    state.drift_score = Some(drift.clone());
                }
                ShadowEvent::StatisticsUpdated(stats) => {
                    state.statistics = stats.clone();
                }
                ShadowEvent::ParityUpdated(parity) => {
                    state.parity_score = Some(parity.clone());
                }
                ShadowEvent::HealthUpdated(health) => {
                    state.health = Some(*health);
                }
                ShadowEvent::ValidatorPromoted { to, .. } => {
                    state.validator.state = *to;
                }
                ShadowEvent::ValidatorDemoted { to, .. } => {
                    state.validator.state = *to;
                }
            }
        }

        state
    }
}
