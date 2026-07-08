use crate::market::state::MarketState;

pub struct StateTransition;

impl StateTransition {
    pub fn next(current: MarketState, target: MarketState) -> Result<MarketState, &'static str> {
        match (current, target) {
            // Cannot transition from Closed directly to Stressed or Dislocated
            (MarketState::Closed, MarketState::Stressed)
            | (MarketState::Closed, MarketState::Dislocated) => {
                Err("Illegal transition from Closed to Stressed/Dislocated")
            }
            // Cannot jump from Healthy directly to Dislocated (must pass through Normal or Stressed)
            (MarketState::Healthy, MarketState::Dislocated) => {
                Err("Illegal transition from Healthy to Dislocated")
            }
            _ => Ok(target),
        }
    }
}
