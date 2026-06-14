use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStatistics {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
}

impl TradeStatistics {
    pub fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            average_win: Decimal::ZERO,
            average_loss: Decimal::ZERO,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
        }
    }
}
