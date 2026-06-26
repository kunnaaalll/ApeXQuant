use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreComponent {
    EventBus,
    MarketData,
    Risk,
    Execution,
    Portfolio,
    Strategy,
    Learning,
    Ai,
    Other(String),
}

impl std::str::FromStr for CoreComponent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "event-bus" => CoreComponent::EventBus,
            "market-data" => CoreComponent::MarketData,
            "risk" => CoreComponent::Risk,
            "execution" => CoreComponent::Execution,
            "portfolio" => CoreComponent::Portfolio,
            "strategy" => CoreComponent::Strategy,
            "learning" => CoreComponent::Learning,
            "ai" => CoreComponent::Ai,
            _ => CoreComponent::Other(s.to_string()),
        })
    }
}

#[derive(Debug)]
pub struct DependencyGraph {
    pub order: Vec<CoreComponent>,
    active_components: HashSet<CoreComponent>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            order: vec![
                CoreComponent::EventBus,
                CoreComponent::MarketData,
                CoreComponent::Risk,
                CoreComponent::Execution,
                CoreComponent::Portfolio,
                CoreComponent::Strategy,
                CoreComponent::Learning,
                CoreComponent::Ai,
            ],
            active_components: HashSet::new(),
        }
    }

    pub fn validate_startup(&self, component: &CoreComponent) -> Result<(), &'static str> {
        let pos = self.order.iter().position(|c| c == component);
        if let Some(idx) = pos {
            for i in 0..idx {
                if !self.active_components.contains(&self.order[i]) {
                    return Err("Missing required dependency for startup");
                }
            }
        }
        Ok(())
    }

    pub fn validate_shutdown(&self, component: &CoreComponent) -> Result<(), &'static str> {
        let pos = self.order.iter().position(|c| c == component);
        if let Some(idx) = pos {
            for i in (idx + 1)..self.order.len() {
                if self.active_components.contains(&self.order[i]) {
                    return Err("Dependent components are still running, cannot shut down yet");
                }
            }
        }
        Ok(())
    }

    pub fn mark_started(&mut self, component: CoreComponent) {
        self.active_components.insert(component);
    }

    pub fn mark_stopped(&mut self, component: &CoreComponent) {
        self.active_components.remove(component);
    }

    pub fn is_running(&self, component: &CoreComponent) -> bool {
        self.active_components.contains(component)
    }
}
