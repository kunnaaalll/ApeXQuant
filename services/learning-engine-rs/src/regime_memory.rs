use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketRegime {
    Trend,
    Range,
    HighVolatility,
    LowVolatility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeRecord {
    pub successful_strategies: Vec<String>,
    pub optimal_symbols: Vec<String>,
    pub max_drawdown_tolerated: Decimal,
}

pub struct RegimeMemory {
    memory: HashMap<MarketRegime, RegimeRecord>,
}

impl Default for RegimeMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl RegimeMemory {
    pub fn new() -> Self {
        let mut memory = HashMap::new();
        memory.insert(
            MarketRegime::Trend,
            RegimeRecord {
                successful_strategies: vec![],
                optimal_symbols: vec![],
                max_drawdown_tolerated: Decimal::new(15, 0),
            },
        );
        memory.insert(
            MarketRegime::Range,
            RegimeRecord {
                successful_strategies: vec![],
                optimal_symbols: vec![],
                max_drawdown_tolerated: Decimal::new(10, 0),
            },
        );
        memory.insert(
            MarketRegime::HighVolatility,
            RegimeRecord {
                successful_strategies: vec![],
                optimal_symbols: vec![],
                max_drawdown_tolerated: Decimal::new(20, 0),
            },
        );
        memory.insert(
            MarketRegime::LowVolatility,
            RegimeRecord {
                successful_strategies: vec![],
                optimal_symbols: vec![],
                max_drawdown_tolerated: Decimal::new(5, 0),
            },
        );

        Self { memory }
    }

    pub fn record_success(&mut self, regime: MarketRegime, strategy_id: String) {
        if let Some(record) = self.memory.get_mut(&regime) {
            if !record.successful_strategies.contains(&strategy_id) {
                record.successful_strategies.push(strategy_id);
            }
        }
    }

    pub fn get_record(&self, regime: &MarketRegime) -> Option<&RegimeRecord> {
        self.memory.get(regime)
    }
}
