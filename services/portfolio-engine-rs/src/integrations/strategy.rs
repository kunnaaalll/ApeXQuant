use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformanceEvent {
    pub strategy_id: String,
    pub symbol: String,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub timestamp: i64,
}

pub struct StrategyClient;

impl Default for StrategyClient {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyClient {
    pub fn new() -> Self {
        Self
    }
}
