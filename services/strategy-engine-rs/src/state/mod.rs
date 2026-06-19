#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StrategyState {
    Dormant,
    Research,
    Emerging,
    Active,
    Strong,
    Elite,
    Weakening,
    Deteriorating,
    Paused,
    Retired,
}

impl StrategyState {
    pub fn can_transition_to(&self, next: &StrategyState) -> bool {
        match (self, next) {
            // Illegal transitions explicitly forbidden
            (StrategyState::Retired, StrategyState::Active) => false,
            (StrategyState::Paused, StrategyState::Elite) => false,
            (StrategyState::Dormant, StrategyState::Strong) => false,
            // Allow all other transitions
            _ => true,
        }
    }
}
