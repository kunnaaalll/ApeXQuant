use super::{WeightOptimizer, WeightType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveState {
    pub weights: HashMap<WeightType, WeightOptimizer>,
}

impl AdaptiveState {
    pub fn new() -> Self {
        Self {
            weights: HashMap::new(),
        }
    }
}

impl Default for AdaptiveState {
    fn default() -> Self {
        Self::new()
    }
}
