//! Performance Metrics — Sharpe, Sortino, Calmar, etc.
//!
//! All computed from a Vec<CompletedTrade>. Uses f64 internally for
//! statistical math (stddev, log), Decimal for all outputs.

use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use crate::trades::CompletedTrade;
use crate::aggregation::PnLAggregate;

#[derive(Debug, Clone)]
pub struct DetailedMetrics {
    /// Sharpe ratio (annualised, 252 trading days, risk-free=0)
    pub sharpe_ratio: Decimal,
    /// Sortino ratio (annualised, only downside deviation)
    pub sortino_ratio: Decimal,
    /// Calmar ratio (CAGR / max drawdown)
    pub calmar_ratio: Decimal,
    /// Maximum drawdown on cumulative PnL curve
    pub max_drawdown: Decimal,
    /// Gross profit / gross loss
    pub profit_factor: Decimal,
    /// Win rate (0–1)
    pub win_rate: Decimal,
    /// Average R-multiple per trade
    pub average_r: Decimal,
    /// Expectancy = win_rate × avg_win − loss_rate × avg_loss
    pub expectancy: Decimal,
    /// Net profit
    pub net_profit: Decimal,
    /// Number of trades
    pub total_trades: u32,
    /// Average holding time in seconds
    pub avg_holding_secs: i64,
    /// Consecutive win/loss streaks (max)
    pub max_win_streak: u32,
    pub max_loss_streak: u32,
}

impl DetailedMetrics {
    pub fn compute(trades: &[CompletedTrade], agg: &PnLAggregate) -> Result<Self, String> {
        if trades.is_empty() {
            return Err("No trades to compute metrics from".to_string());
        }
        let pnls: Vec<f64> = trades.iter()
            .map(|t| t.net_pnl.to_f64().unwrap_or(0.0))
            .collect();

        let max_drawdown = max_drawdown_f64(&pnls);
        let (sharpe, sortino) = sharpe_sortino(&pnls);
        let net_profit_f = pnls.iter().sum::<f64>();

        // Calmar = net_profit / max_drawdown (not CAGR without time)
        let calmar_ratio = if max_drawdown.abs() > 1e-12 {
            Decimal::from_f64(net_profit_f / max_drawdown)
                .unwrap_or(Decimal::ZERO)
                .round_dp(4)
        } else {
            Decimal::ZERO
        };

        // Expectancy
        let avg_win = if agg.winning_trades > 0 {
            agg.gross_profit / Decimal::from(agg.winning_trades)
        } else {
            Decimal::ZERO
        };
        let avg_loss = if agg.losing_trades > 0 {
            agg.gross_loss / Decimal::from(agg.losing_trades)
        } else {
            Decimal::ZERO
        };
        let loss_rate = Decimal::ONE - agg.win_rate();
        let expectancy = agg.win_rate() * avg_win - loss_rate * avg_loss;

        let (max_win_streak, max_loss_streak) = compute_streaks(trades);

        Ok(Self {
            sharpe_ratio: Decimal::from_f64(sharpe).unwrap_or(Decimal::ZERO).round_dp(4),
            sortino_ratio: Decimal::from_f64(sortino).unwrap_or(Decimal::ZERO).round_dp(4),
            calmar_ratio,
            max_drawdown: Decimal::from_f64(max_drawdown).unwrap_or(Decimal::ZERO).round_dp(2),
            profit_factor: agg.profit_factor(),
            win_rate: agg.win_rate(),
            average_r: agg.average_r(),
            expectancy,
            net_profit: agg.net_profit,
            total_trades: agg.total_trades,
            avg_holding_secs: agg.average_holding_secs(),
            max_win_streak,
            max_loss_streak,
        })
    }
}

/// Annualised Sharpe and Sortino from daily PnL series.
/// √252 annualisation factor (trading days).
fn sharpe_sortino(pnls: &[f64]) -> (f64, f64) {
    let n = pnls.len() as f64;
    if n < 2.0 { return (0.0, 0.0); }
    let mean = pnls.iter().sum::<f64>() / n;
    let var = pnls.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = var.sqrt();
    let sharpe = if std > 1e-12 { (mean / std) * 252_f64.sqrt() } else { 0.0 };

    // Sortino: downside deviation (only negative returns count)
    let down_var = pnls.iter()
        .filter(|&&x| x < 0.0)
        .map(|&x| x.powi(2))
        .sum::<f64>() / n;
    let down_std = down_var.sqrt();
    let sortino = if down_std > 1e-12 { (mean / down_std) * 252_f64.sqrt() } else { 0.0 };

    (sharpe, sortino)
}

/// Maximum drawdown from cumulative PnL (returns positive dollar amount).
fn max_drawdown_f64(pnls: &[f64]) -> f64 {
    let mut cumulative = 0.0_f64;
    let mut peak = 0.0_f64;
    let mut max_dd = 0.0_f64;
    for &p in pnls {
        cumulative += p;
        if cumulative > peak { peak = cumulative; }
        let dd = peak - cumulative;
        if dd > max_dd { max_dd = dd; }
    }
    max_dd
}

/// Compute maximum consecutive win and loss streaks.
fn compute_streaks(trades: &[CompletedTrade]) -> (u32, u32) {
    let mut max_win = 0u32;
    let mut max_loss = 0u32;
    let mut win_streak = 0u32;
    let mut loss_streak = 0u32;

    for t in trades {
        if t.is_winner() {
            win_streak += 1;
            loss_streak = 0;
        } else {
            loss_streak += 1;
            win_streak = 0;
        }
        if win_streak > max_win { max_win = win_streak; }
        if loss_streak > max_loss { max_loss = loss_streak; }
    }
    (max_win, max_loss)
}
