use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PortfolioMetrics {
    pub total_return: Decimal,
    pub net_return: Decimal,
    pub gross_return: Decimal,

    pub average_rr: Decimal,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,

    pub win_rate: Decimal,
    pub loss_rate: Decimal,

    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,

    pub average_duration: Duration,
    pub average_holding_efficiency: Decimal,
    pub capital_efficiency: Decimal,

    pub recovery_factor: Decimal,
    pub ulcer_index: Decimal,

    pub max_drawdown: Decimal,
    pub current_drawdown: Decimal,
    pub time_under_water: Duration,
}

impl PortfolioMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enforce the APEX invariant: Profit factor >= 0
    pub fn enforce_invariants(&mut self) {
        if self.profit_factor < Decimal::ZERO {
            self.profit_factor = Decimal::ZERO;
        }
        if self.win_rate < Decimal::ZERO || self.win_rate > Decimal::ONE {
            self.win_rate = Decimal::ZERO;
        }
        if self.loss_rate < Decimal::ZERO || self.loss_rate > Decimal::ONE {
            self.loss_rate = Decimal::ZERO;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetricsSnapshot {
    pub timestamp: i64,
    pub version: u64,
    pub metrics: PortfolioMetrics,
}
