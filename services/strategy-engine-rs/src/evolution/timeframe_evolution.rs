use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeEvolution {
    pub timeframe: String,
    pub generation: u32,
    pub score: f64,
}

impl TimeframeEvolution {
    pub fn new(timeframe: String, score: f64) -> Self {
        Self {
            timeframe,
            generation: 1,
            score,
        }
    }
}
