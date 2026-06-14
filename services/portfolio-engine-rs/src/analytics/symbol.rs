// src/analytics/symbol.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SymbolPerformanceMetrics {
    pub profit_factor: f64,
    pub expectancy: f64,
    pub win_rate: f64,
    pub average_rr: f64,
    pub total_trades: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SymbolPerformanceProfile {
    /// Maps symbol ticker (e.g., "BTCUSDT") to its performance metrics
    pub symbol_metrics: HashMap<String, SymbolPerformanceMetrics>,
}

impl SymbolPerformanceProfile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_metrics(&self, symbol: &str) -> Option<&SymbolPerformanceMetrics> {
        self.symbol_metrics.get(symbol)
    }

    pub fn update_metrics(&mut self, symbol: String, metrics: SymbolPerformanceMetrics) {
        self.symbol_metrics.insert(symbol, metrics);
    }
}
