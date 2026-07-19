use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// A single completed trade record for replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub trade_id: String,
    pub session: String,
    pub regime: String,
    pub symbol: String,
    pub timeframe: String,
    pub pattern_id: String,
    pub sl: Decimal,
    pub tp: Decimal,
    pub rr: Decimal,
    pub entry_quality: Decimal,
    /// Actual P&L in R-multiples (+1R, -1R, etc.)
    pub r_outcome: Decimal,
    /// True if the actual outcome was a win
    pub is_win: bool,
}

/// A full historical replay result over a filtered subset of trades.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub trade_count: u32,
    pub total_r: Decimal,
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
    pub explanation: String,
}

impl ReplayResult {
    pub fn empty(reason: &str) -> Self {
        Self {
            trade_count: 0,
            total_r: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            explanation: reason.to_string(),
        }
    }
}

/// Filter specification for trade selection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayFilter {
    pub session: Option<String>,
    pub regime: Option<String>,
    pub symbol: Option<String>,
    pub timeframe: Option<String>,
    pub pattern_id: Option<String>,
    pub min_rr: Option<Decimal>,
    pub min_entry_quality: Option<Decimal>,
}

/// Deterministic historical replay engine.
/// No randomness. No predictions. Historical data only.
pub struct ReplayEngine;

impl ReplayEngine {
    /// Replay a filtered subset of historical trades and compute aggregate metrics.
    pub fn replay(trades: &[TradeRecord], filter: &ReplayFilter) -> ReplayResult {
        let filtered: Vec<&TradeRecord> = trades
            .iter()
            .filter(|t| Self::passes_filter(t, filter))
            .collect();

        if filtered.is_empty() {
            return ReplayResult::empty("No trades matched the replay filter.");
        }

        let trade_count = filtered.len() as u32;
        let total_r: Decimal = filtered.iter().map(|t| t.r_outcome).sum();
        let wins = filtered.iter().filter(|t| t.is_win).count();
        let win_rate = if trade_count > 0 {
            Decimal::from(wins) / Decimal::from(trade_count)
        } else {
            Decimal::ZERO
        };
        let expectancy = if trade_count > 0 {
            total_r / Decimal::from(trade_count)
        } else {
            Decimal::ZERO
        };

        let gross_profit: Decimal = filtered
            .iter()
            .filter(|t| t.r_outcome > Decimal::ZERO)
            .map(|t| t.r_outcome)
            .sum();
        let gross_loss: Decimal = filtered
            .iter()
            .filter(|t| t.r_outcome < Decimal::ZERO)
            .map(|t| t.r_outcome.abs())
            .sum();

        let profit_factor = if gross_loss > Decimal::ZERO {
            gross_profit / gross_loss
        } else if gross_profit > Decimal::ZERO {
            dec!(999) // infinite edge proxy — capped
        } else {
            Decimal::ZERO
        };

        let max_drawdown = Self::compute_max_drawdown(&filtered);

        let explanation = format!(
            "Replay: {} trades | Win rate: {:.2}% | Expectancy: {:.3}R | PF: {:.3} | Max DD: {:.2}%",
            trade_count,
            win_rate * dec!(100),
            expectancy,
            profit_factor,
            max_drawdown * dec!(100),
        );

        ReplayResult {
            trade_count,
            total_r,
            win_rate,
            expectancy,
            profit_factor,
            max_drawdown,
            explanation,
        }
    }

    fn passes_filter(trade: &TradeRecord, filter: &ReplayFilter) -> bool {
        if let Some(ref s) = filter.session {
            if &trade.session != s {
                return false;
            }
        }
        if let Some(ref r) = filter.regime {
            if &trade.regime != r {
                return false;
            }
        }
        if let Some(ref sym) = filter.symbol {
            if &trade.symbol != sym {
                return false;
            }
        }
        if let Some(ref tf) = filter.timeframe {
            if &trade.timeframe != tf {
                return false;
            }
        }
        if let Some(ref p) = filter.pattern_id {
            if &trade.pattern_id != p {
                return false;
            }
        }
        if let Some(min_rr) = filter.min_rr {
            if trade.rr < min_rr {
                return false;
            }
        }
        if let Some(min_eq) = filter.min_entry_quality {
            if trade.entry_quality < min_eq {
                return false;
            }
        }
        true
    }

    fn compute_max_drawdown(trades: &[&TradeRecord]) -> Decimal {
        let mut peak = Decimal::ZERO;
        let mut equity = Decimal::ZERO;
        let mut max_dd = Decimal::ZERO;

        for trade in trades {
            equity += trade.r_outcome;
            if equity > peak {
                peak = equity;
            }
            let dd = if peak > Decimal::ZERO {
                (peak - equity) / peak
            } else {
                Decimal::ZERO
            };
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd
    }
}
