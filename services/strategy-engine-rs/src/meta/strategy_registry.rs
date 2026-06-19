use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct StrategyRegistry {
    _strategies: HashMap<String, String>, // Placeholder implementation
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            _strategies: HashMap::new(),
        }
    }
}
