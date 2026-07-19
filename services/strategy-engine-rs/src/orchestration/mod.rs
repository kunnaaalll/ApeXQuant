use crate::strategy::StrategyProfile;

#[derive(Debug, Clone, Default)]
pub struct StrategyCoordinator {
    pub active_strategies: Vec<StrategyProfile>,
    pub paused_strategies: Vec<StrategyProfile>,
    pub retired_strategies: Vec<StrategyProfile>,
}

impl StrategyCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self, strategy: StrategyProfile) {
        self.active_strategies.push(strategy);
    }

    pub fn disable(&mut self, strategy_id: &str) {
        if let Some(pos) = self
            .active_strategies
            .iter()
            .position(|s| s.strategy_id == strategy_id)
        {
            let strategy = self.active_strategies.remove(pos);
            self.paused_strategies.push(strategy);
        }
    }

    pub fn retire(&mut self, strategy_id: &str) {
        if let Some(pos) = self
            .active_strategies
            .iter()
            .position(|s| s.strategy_id == strategy_id)
        {
            let strategy = self.active_strategies.remove(pos);
            self.retired_strategies.push(strategy);
        } else if let Some(pos) = self
            .paused_strategies
            .iter()
            .position(|s| s.strategy_id == strategy_id)
        {
            let strategy = self.paused_strategies.remove(pos);
            self.retired_strategies.push(strategy);
        }
    }

    pub fn promote(&mut self) {
        // To be implemented
    }

    pub fn demote(&mut self) {
        // To be implemented
    }
}
