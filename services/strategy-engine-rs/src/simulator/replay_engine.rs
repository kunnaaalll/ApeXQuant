use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEngine {
    pub active: bool,
}

impl ReplayEngine {
    pub fn new() -> Self {
        Self { active: true }
    }
}

impl Default for ReplayEngine {
    fn default() -> Self {
        Self::new()
    }
}
