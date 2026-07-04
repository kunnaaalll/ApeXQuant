//! Daily / Weekly / Monthly Statistics — time-bucketed aggregations.

use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::trades::CompletedTrade;

/// Rolling statistics for a single time bucket (day/week/month).
#[derive(Debug, Clone, Default)]
pub struct TimeBucketStats {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub net_pnl: Decimal,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub max_single_win: Decimal,
    pub max_single_loss: Decimal,
}

impl TimeBucketStats {
    pub fn win_rate(&self) -> Decimal {
        if self.total_trades == 0 { return Decimal::ZERO; }
        Decimal::from(self.winning_trades) / Decimal::from(self.total_trades)
    }

    pub fn ingest(&mut self, trade: &CompletedTrade) {
        self.total_trades += 1;
        self.net_pnl += trade.net_pnl;
        if trade.net_pnl > Decimal::ZERO {
            self.winning_trades += 1;
            self.gross_profit += trade.net_pnl;
            if trade.net_pnl > self.max_single_win {
                self.max_single_win = trade.net_pnl;
            }
        } else {
            self.gross_loss += trade.net_pnl.abs();
            if trade.net_pnl.abs() > self.max_single_loss.abs() {
                self.max_single_loss = trade.net_pnl;
            }
        }
    }
}

/// Bucket resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketResolution {
    Daily,   // YYYY-MM-DD
    Weekly,  // YYYY-WW
    Monthly, // YYYY-MM
}

/// Partition a trade stream into time-bucketed statistics.
pub fn bucket_trades(
    trades: &[CompletedTrade],
    resolution: BucketResolution,
) -> HashMap<String, TimeBucketStats> {
    let mut buckets: HashMap<String, TimeBucketStats> = HashMap::new();

    for trade in trades {
        let key = bucket_key(trade.exit_time_ms, resolution);
        buckets.entry(key).or_default().ingest(trade);
    }

    buckets
}

fn bucket_key(exit_ms: i64, resolution: BucketResolution) -> String {
    let secs = exit_ms / 1000;
    let days_since_epoch = secs / 86400;
    let total_secs = secs;

    // Simple calendar computation (no chrono dependency here)
    let year = days_to_year(days_since_epoch);
    let day_of_year = days_since_epoch - year_start_day(year);

    match resolution {
        BucketResolution::Daily => {
            let (month, day) = day_of_year_to_month_day(day_of_year, is_leap(year));
            format!("{:04}-{:02}-{:02}", year, month, day)
        }
        BucketResolution::Weekly => {
            let week = day_of_year / 7;
            format!("{:04}-W{:02}", year, week)
        }
        BucketResolution::Monthly => {
            let (month, _) = day_of_year_to_month_day(day_of_year, is_leap(year));
            format!("{:04}-{:02}", year, month)
        }
    }
}

fn days_to_year(days: i64) -> i64 {
    // Approx: 400-year cycle = 146097 days
    let mut y = 1970 + (days / 365);
    while year_start_day(y + 1) <= days { y += 1; }
    while year_start_day(y) > days { y -= 1; }
    y
}

fn year_start_day(y: i64) -> i64 {
    let y0 = y - 1970;
    y0 * 365 + y0 / 4 - y0 / 100 + y0 / 400
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn day_of_year_to_month_day(doy: i64, leap: bool) -> (i64, i64) {
    let month_days = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut remaining = doy;
    let mut month = 1i64;
    for &days in &month_days {
        if remaining < days {
            break;
        }
        remaining -= days;
        month += 1;
    }
    (month, remaining + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trade_at(exit_ms: i64, pnl: i64) -> CompletedTrade {
        CompletedTrade {
            trade_id: "t".to_string(),
            account_id: "a".to_string(),
            symbol: "EURUSD".to_string(),
            strategy_id: "s".to_string(),
            is_long: pnl > 0,
            entry_price: Decimal::ZERO,
            exit_price: Decimal::ZERO,
            quantity: Decimal::ONE,
            commission: Decimal::ZERO,
            swap: Decimal::ZERO,
            gross_pnl: Decimal::new(pnl, 0),
            net_pnl: Decimal::new(pnl, 0),
            entry_time_ms: exit_ms - 3600_000,
            exit_time_ms: exit_ms,
            mae: Decimal::ZERO,
            mfe: Decimal::ZERO,
            r_multiple: Decimal::ZERO,
        }
    }

    #[test]
    fn test_daily_bucketing() {
        // Same epoch-day: 1970-01-01 (ms 0) and 1970-01-01 (ms 3600)
        let trades = vec![
            trade_at(3_600_000, 100),  // Jan 1 1970
            trade_at(3_600_000, -50),  // Jan 1 1970
        ];
        let buckets = bucket_trades(&trades, BucketResolution::Daily);
        assert_eq!(buckets.len(), 1, "same day → same bucket");
        let (_, stats) = buckets.iter().next().expect("no bucket");
        assert_eq!(stats.total_trades, 2);
    }
}
