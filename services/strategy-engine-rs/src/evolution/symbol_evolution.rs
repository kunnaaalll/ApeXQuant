use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEvolution {
    pub symbol: String,
    pub generation: u32,
    pub score: f64,
}

impl SymbolEvolution {
    pub fn new(symbol: String, score: f64) -> Self {
        Self {
            symbol,
            generation: 1,
            score,
        }
    }
}
