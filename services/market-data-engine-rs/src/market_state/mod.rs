#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketState {
    Healthy,
    Warning,
    Stressed,
    Dislocated,
    Broken,
}

pub struct MarketStateEngine;

impl MarketStateEngine {
    pub fn transition(current: MarketState, target: MarketState) -> Result<MarketState, &'static str> {
        if current == target {
            return Ok(current);
        }

        match (current, target) {
            (MarketState::Broken, MarketState::Healthy) => Err("Illegal jump: Broken to Healthy"),
            (MarketState::Broken, MarketState::Warning) => Err("Illegal jump: Broken to Warning"),
            (MarketState::Broken, MarketState::Stressed) => Err("Illegal jump: Broken to Stressed"),
            (MarketState::Dislocated, MarketState::Healthy) => Err("Illegal jump: Dislocated to Healthy"),
            _ => Ok(target),
        }
    }
}
