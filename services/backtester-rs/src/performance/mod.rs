//! Performance Metrics Module
//!
//! Calculates all key quantitative trading metrics from a slice of trade results.
//! All arithmetic uses `rust_decimal::Decimal` — zero float business logic.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// A single completed trade.
#[derive(Debug, Clone)]
pub struct TradeRecord {
    /// Net PnL in account currency (after commissions, swaps)
    pub pnl: Decimal,
    /// R-multiple: PnL / initial risk (stop distance × position size)
    pub r_multiple: Decimal,
    /// Maximum adverse excursion (worst drawdown within the trade)
    pub mae: Decimal,
}

/// Complete performance metrics computed from a trade history.
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    /// Percentage of trades that are winners (0–1)
    pub win_rate: Decimal,
    /// Total gross profit (sum of positive PnL)
    pub gross_profit: Decimal,
    /// Total gross loss (absolute sum of negative PnL)
    pub gross_loss: Decimal,
    /// Gross profit / gross loss
    pub profit_factor: Decimal,
    /// Mean R-multiple across all trades
    pub average_r: Decimal,
    /// (win_rate × avg_win) − (loss_rate × avg_loss)
    pub expectancy: Decimal,
    /// Largest peak-to-trough drawdown on cumulative PnL curve (positive = loss)
    pub max_drawdown: Decimal,
    /// Net profit / max_drawdown
    pub recovery_factor: Decimal,
    /// CAGR (annualized) / max_drawdown — approximated as net_profit / max_drawdown for non-time series
    pub mar_ratio: Decimal,
    /// Total net profit
    pub net_profit: Decimal,
}

impl PerformanceMetrics {
    /// Compute all metrics from a slice of trade records.
    ///
    /// Returns an error if trades is empty.
    pub fn calculate(trades: &[TradeRecord]) -> Result<Self, String> {
        if trades.is_empty() {
            return Err("Cannot compute metrics from empty trade history".to_string());
        }

        let total_trades = trades.len() as u32;

        // Partition wins/losses
        let winning_trades = trades.iter().filter(|t| t.pnl > Decimal::ZERO).count() as u32;
        let losing_trades = total_trades - winning_trades;

        let gross_profit: Decimal = trades.iter()
            .filter(|t| t.pnl > Decimal::ZERO)
            .map(|t| t.pnl)
            .sum();

        let gross_loss: Decimal = trades.iter()
            .filter(|t| t.pnl < Decimal::ZERO)
            .map(|t| t.pnl.abs())
            .sum();

        let net_profit: Decimal = trades.iter().map(|t| t.pnl).sum();

        // Win rate
        let win_rate = Decimal::from(winning_trades) / Decimal::from(total_trades);

        // Profit factor
        let profit_factor = if gross_loss == Decimal::ZERO {
            if gross_profit > Decimal::ZERO { Decimal::new(999, 0) } else { Decimal::ZERO }
        } else {
            gross_profit / gross_loss
        };

        // Average R
        let r_sum: Decimal = trades.iter().map(|t| t.r_multiple).sum();
        let average_r = r_sum / Decimal::from(total_trades);

        // Expectancy
        let avg_win = if winning_trades > 0 {
            gross_profit / Decimal::from(winning_trades)
        } else {
            Decimal::ZERO
        };
        let avg_loss = if losing_trades > 0 {
            gross_loss / Decimal::from(losing_trades)
        } else {
            Decimal::ZERO
        };
        let loss_rate = Decimal::ONE - win_rate;
        let expectancy = (win_rate * avg_win) - (loss_rate * avg_loss);

        // Max drawdown from cumulative PnL curve
        let max_drawdown = compute_max_drawdown(trades);

        // Recovery factor
        let recovery_factor = if max_drawdown == Decimal::ZERO {
            Decimal::ZERO
        } else {
            net_profit / max_drawdown
        };

        // MAR ratio (net profit / max drawdown as proxy — proper CAGR requires time data)
        let mar_ratio = recovery_factor; // same calculation without time normalization

        Ok(PerformanceMetrics {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            gross_profit,
            gross_loss,
            profit_factor,
            average_r,
            expectancy,
            max_drawdown,
            recovery_factor,
            mar_ratio,
            net_profit,
        })
    }
}

/// Compute maximum peak-to-trough drawdown on cumulative PnL curve.
/// Returns the drawdown in absolute currency terms (always positive).
fn compute_max_drawdown(trades: &[TradeRecord]) -> Decimal {
    let mut cumulative = Decimal::ZERO;
    let mut peak = Decimal::ZERO;
    let mut max_dd = Decimal::ZERO;

    for trade in trades {
        cumulative += trade.pnl;
        if cumulative > peak {
            peak = cumulative;
        }
        let dd = peak - cumulative;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    max_dd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trades(data: &[(i64, i64)]) -> Vec<TradeRecord> {
        data.iter().map(|&(pnl, r_mul)| TradeRecord {
            pnl: Decimal::new(pnl, 0),
            r_multiple: Decimal::new(r_mul, 1), // r_mul tenths
            mae: Decimal::ZERO,
        }).collect()
    }

    #[test]
    fn test_win_rate() {
        // 6 wins, 4 losses
        let trades = make_trades(&[
            (100, 10), (200, 20), (-50, -5), (150, 15),
            (-80, -8), (120, 12), (-30, -3), (90, 9),
            (-40, -4), (110, 11),
        ]);
        let m = PerformanceMetrics::calculate(&trades).expect("failed");
        assert_eq!(m.winning_trades, 6);
        assert_eq!(m.losing_trades, 4);
        assert_eq!(m.win_rate, Decimal::new(6, 0) / Decimal::new(10, 0));
    }

    #[test]
    fn test_profit_factor() {
        let trades = make_trades(&[(300, 30), (-100, -10)]);
        let m = PerformanceMetrics::calculate(&trades).expect("failed");
        // gross_profit=300, gross_loss=100, PF=3
        assert_eq!(m.profit_factor, Decimal::new(3, 0));
    }

    #[test]
    fn test_max_drawdown() {
        // Cumulative: 100, 200, 150, 300, 250 → peak 300, trough 250 → dd=50
        let trades = make_trades(&[(100, 10), (100, 10), (-50, -5), (150, 15), (-50, -5)]);
        let m = PerformanceMetrics::calculate(&trades).expect("failed");
        assert_eq!(m.max_drawdown, Decimal::new(50, 0));
    }

    #[test]
    fn test_expectancy_positive() {
        // win_rate=0.6, avg_win=100, avg_loss=50 → expectancy=(0.6×100)-(0.4×50)=40
        let trades = make_trades(&[
            (100, 10), (100, 10), (100, 10), (100, 10), (100, 10), (100, 10),
            (-50, -5), (-50, -5), (-50, -5), (-50, -5),
        ]);
        let m = PerformanceMetrics::calculate(&trades).expect("failed");
        assert_eq!(m.expectancy, Decimal::new(40, 0));
    }

    #[test]
    fn test_empty_trades_error() {
        let result = PerformanceMetrics::calculate(&[]);
        assert!(result.is_err());
    }
}
