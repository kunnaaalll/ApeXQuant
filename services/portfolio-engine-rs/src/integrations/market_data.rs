use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRegimeEvent {
    pub symbol: String,
    pub regime: String,
    pub confidence: f32,
    pub timestamp: i64,
}

pub struct MarketDataClient;

impl Default for MarketDataClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketDataClient {
    pub fn new() -> Self {
        Self
    }
}
