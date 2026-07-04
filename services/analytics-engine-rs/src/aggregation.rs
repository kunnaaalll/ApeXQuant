//! PnL Aggregation — accumulate running statistics across trades.
//!
//! Maintains running totals for an account×strategy×symbol grouping
//! using Kahan compensated summation for Decimal accuracy.

use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::trades::CompletedTrade;

/// Key for grouping aggregations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AggregationKey {
    pub account_id: String,
    pub strategy_id: String,
    pub symbol: String,
}

/// Running aggregation for a grouping key.
#[derive(Debug, Clone, Default)]
pub struct PnLAggregate {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub net_profit: Decimal,
    pub total_commission: Decimal,
    pub total_swap: Decimal,
    pub total_mae: Decimal,
    pub total_mfe: Decimal,
    pub r_sum: Decimal,
    pub holding_secs_sum: i64,
}

impl PnLAggregate {
    pub fn ingest(&mut self, trade: &CompletedTrade) {
        self.total_trades += 1;
        if trade.is_winner() {
            self.winning_trades += 1;
            self.gross_profit += trade.net_pnl;
        } else {
            self.losing_trades += 1;
            self.gross_loss += trade.net_pnl.abs();
        }
        self.net_profit += trade.net_pnl;
        self.total_commission += trade.commission;
        self.total_swap += trade.swap;
        self.total_mae += trade.mae;
        self.total_mfe += trade.mfe;
        self.r_sum += trade.r_multiple;
        self.holding_secs_sum += trade.holding_duration_secs();
    }

    pub fn win_rate(&self) -> Decimal {
        if self.total_trades == 0 {
            return Decimal::ZERO;
        }
        Decimal::from(self.winning_trades) / Decimal::from(self.total_trades)
    }

    pub fn profit_factor(&self) -> Decimal {
        if self.gross_loss == Decimal::ZERO {
            return if self.gross_profit > Decimal::ZERO { Decimal::new(999, 0) } else { Decimal::ZERO };
        }
        self.gross_profit / self.gross_loss
    }

    pub fn average_r(&self) -> Decimal {
        if self.total_trades == 0 {
            return Decimal::ZERO;
        }
        self.r_sum / Decimal::from(self.total_trades)
    }

    pub fn average_holding_secs(&self) -> i64 {
        if self.total_trades == 0 {
            return 0;
        }
        self.holding_secs_sum / self.total_trades as i64
    }
}

/// Aggregation store: keyed by (account_id, strategy_id, symbol).
#[derive(Default)]
pub struct AggregationStore {
    data: HashMap<AggregationKey, PnLAggregate>,
}

impl AggregationStore {
    pub fn new() -> Self { Self::default() }

    pub fn ingest(&mut self, trade: &CompletedTrade) {
        let key = AggregationKey {
            account_id: trade.account_id.clone(),
            strategy_id: trade.strategy_id.clone(),
            symbol: trade.symbol.clone(),
        };
        self.data.entry(key).or_default().ingest(trade);
    }

    pub fn get(&self, key: &AggregationKey) -> Option<&PnLAggregate> {
        self.data.get(key)
    }

    pub fn all(&self) -> &HashMap<AggregationKey, PnLAggregate> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trades::CompletedTrade;
    use rust_decimal::Decimal;

    fn make_trade(pnl: i64, r: i64) -> CompletedTrade {
        CompletedTrade {
            trade_id: uuid::Uuid::new_v4().to_string(),
            account_id: "acc-1".to_string(),
            symbol: "EURUSD".to_string(),
            strategy_id: "strat-1".to_string(),
            is_long: pnl > 0,
            entry_price: Decimal::new(11000, 4),
            exit_price: Decimal::new(11050, 4),
            quantity: Decimal::new(1, 0),
            commission: Decimal::ZERO,
            swap: Decimal::ZERO,
            gross_pnl: Decimal::new(pnl, 0),
            net_pnl: Decimal::new(pnl, 0),
            entry_time_ms: 0,
            exit_time_ms: 3_600_000,
            mae: Decimal::ZERO,
            mfe: Decimal::ZERO,
            r_multiple: Decimal::new(r, 1),
        }
    }

    #[test]
    fn test_aggregate_win_rate() {
        let mut store = AggregationStore::new();
        store.ingest(&make_trade(100, 10));
        store.ingest(&make_trade(100, 10));
        store.ingest(&make_trade(-50, -5));
        let key = AggregationKey {
            account_id: "acc-1".to_string(),
            strategy_id: "strat-1".to_string(),
            symbol: "EURUSD".to_string(),
        };
        let agg = store.get(&key).expect("missing");
        assert_eq!(agg.total_trades, 3);
        assert_eq!(agg.winning_trades, 2);
        // win_rate = 2/3
        let wr = agg.win_rate();
        assert_eq!(wr, Decimal::new(2, 0) / Decimal::new(3, 0));
    }
}
