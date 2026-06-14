use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PortfolioMetrics {
    pub total_return: f64,
    pub net_return: f64,
    pub gross_return: f64,
    
    pub average_rr: f64,
    pub profit_factor: f64,
    pub expectancy: f64,
    
    pub win_rate: f64,
    pub loss_rate: f64,
    
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    
    pub average_duration: Duration,
    pub average_holding_efficiency: f64,
    pub capital_efficiency: f64,
    
    pub recovery_factor: f64,
    pub ulcer_index: f64,
    
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub time_under_water: Duration,
}

impl PortfolioMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enforce the APEX invariant: Profit factor >= 0
    pub fn enforce_invariants(&mut self) {
        if self.profit_factor < 0.0 || self.profit_factor.is_nan() {
            self.profit_factor = 0.0;
        }
        if self.win_rate < 0.0 || self.win_rate > 1.0 || self.win_rate.is_nan() {
            self.win_rate = 0.0;
        }
        if self.loss_rate < 0.0 || self.loss_rate > 1.0 || self.loss_rate.is_nan() {
            self.loss_rate = 0.0;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetricsSnapshot {
    pub timestamp: i64,
    pub version: u64,
    pub metrics: PortfolioMetrics,
}
