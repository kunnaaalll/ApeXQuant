use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::TradeStatistics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionStatistics {
    pub average_duration_secs: u64,
    pub win_rate: f32,
    pub profit_factor: Decimal,
    /// Expectancy = (win_rate × avg_win) − (loss_rate × avg_loss)
    pub expectancy: Decimal,
}

impl PositionStatistics {
    /// Derive position-level statistics from pre-computed trade statistics.
    pub fn from_trade_stats(stats: &TradeStatistics, average_duration_secs: u64) -> Self {
        let win_rate_dec = Decimal::from_f32_retain(stats.win_rate).unwrap_or(Decimal::ZERO);
        let loss_rate_dec = Decimal::ONE - win_rate_dec;
        let expectancy = (win_rate_dec * stats.average_win) - (loss_rate_dec * stats.average_loss);

        Self {
            average_duration_secs,
            win_rate: stats.win_rate,
            profit_factor: stats.profit_factor,
            expectancy,
        }
    }
}
