use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStatistics {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f32,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub profit_factor: Decimal,
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
}

impl TradeStatistics {
    /// Build statistics from a slice of realized PnL values (one per closed trade).
    pub fn from_pnl_list(pnl_list: &[Decimal]) -> Self {
        if pnl_list.is_empty() {
            return Self {
                total_trades: 0,
                winning_trades: 0,
                losing_trades: 0,
                win_rate: 0.0,
                average_win: Decimal::ZERO,
                average_loss: Decimal::ZERO,
                profit_factor: Decimal::ZERO,
                max_consecutive_wins: 0,
                max_consecutive_losses: 0,
            };
        }

        let mut winning = 0u32;
        let mut losing = 0u32;
        let mut gross_win = Decimal::ZERO;
        let mut gross_loss = Decimal::ZERO;
        let mut streak = 0i32;
        let mut max_wins = 0u32;
        let mut max_losses = 0u32;

        for &pnl in pnl_list {
            if pnl > Decimal::ZERO {
                winning += 1;
                gross_win += pnl;
                streak = if streak > 0 { streak + 1 } else { 1 };
                if streak as u32 > max_wins {
                    max_wins = streak as u32;
                }
            } else if pnl < Decimal::ZERO {
                losing += 1;
                gross_loss += pnl.abs();
                streak = if streak < 0 { streak - 1 } else { -1 };
                if streak.unsigned_abs() > max_losses {
                    max_losses = streak.unsigned_abs();
                }
            }
        }

        let total = pnl_list.len() as u32;
        let average_win = if winning > 0 {
            gross_win / Decimal::from(winning)
        } else {
            Decimal::ZERO
        };
        let average_loss = if losing > 0 {
            gross_loss / Decimal::from(losing)
        } else {
            Decimal::ZERO
        };
        let profit_factor = if gross_loss > Decimal::ZERO {
            gross_win / gross_loss
        } else {
            Decimal::ZERO
        };
        let win_rate = winning as f32 / total as f32;

        Self {
            total_trades: total,
            winning_trades: winning,
            losing_trades: losing,
            win_rate,
            average_win,
            average_loss,
            profit_factor,
            max_consecutive_wins: max_wins,
            max_consecutive_losses: max_losses,
        }
    }
}
