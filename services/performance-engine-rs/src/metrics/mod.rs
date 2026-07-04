use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeMetrics {
    pub total_trades: u32,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub net_profit: Decimal,
}

impl TradeMetrics {
    pub fn calculate(
        wins: &[Decimal],
        losses: &[Decimal],
    ) -> Self {
        let total_trades = (wins.len() + losses.len()) as u32;
        if total_trades == 0 {
            return Self::default();
        }

        let win_rate = Decimal::from(wins.len()) / Decimal::from(total_trades);
        
        let gross_wins: Decimal = wins.iter().sum();
        let gross_losses: Decimal = losses.iter().sum();
        
        let profit_factor = if gross_losses == Decimal::ZERO {
            dec!(99.0) // high cap
        } else {
            gross_wins / gross_losses.abs()
        };

        Self {
            total_trades,
            win_rate,
            profit_factor,
            net_profit: gross_wins + gross_losses,
        }
    }
}
