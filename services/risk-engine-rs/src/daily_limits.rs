//! Daily limit tracking and enforcement
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// State of daily limits
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DailyLimitState {
    /// Normal operation
    Normal,
    /// Approaching daily limit
    NearLimit {
        /// Remaining risk as percentage of limit
        remaining_pct: Decimal,
    },
    /// Daily limit reached - no new positions
    LimitReached,
}

impl DailyLimitState {
    /// Whether new positions should be blocked
    pub fn blocks_new_positions(&self) -> bool {
        matches!(self, DailyLimitState::LimitReached)
    }

    /// Get the appropriate position size reduction
    pub fn position_scaling(&self) -> Decimal {
        match self {
            DailyLimitState::Normal => Decimal::ONE,
            DailyLimitState::NearLimit { remaining_pct } => *remaining_pct,
            DailyLimitState::LimitReached => Decimal::ZERO,
        }
    }
}

impl Default for DailyLimitState {
    fn default() -> Self {
        DailyLimitState::Normal
    }
}

/// Daily limit tracking engine
pub struct DailyLimitsEngine {
    /// Maximum daily loss as percentage of equity
    daily_loss_limit_percent: Decimal,
    /// Maximum trades per day
    max_daily_trades: u32,
    /// Warning threshold at 80% of limit
    warning_threshold: Decimal,
    /// Today's tracked PnL
    today_pnl: Decimal,
    /// Today's trade count
    today_trades: u32,
    /// Current equity (for calculations)
    current_equity: Decimal,
    /// Last reset date
    last_reset: OffsetDateTime,
}

impl DailyLimitsEngine {
    /// Create new daily limits engine
    pub fn new(daily_loss_limit_percent: Decimal, max_daily_trades: u32) -> Self {
        Self {
            daily_loss_limit_percent,
            max_daily_trades,
            warning_threshold: Decimal::from_f64(0.8).unwrap_or(Decimal::ONE),
            today_pnl: Decimal::ZERO,
            today_trades: 0,
            current_equity: Decimal::ZERO,
            last_reset: OffsetDateTime::now_utc(),
        }
    }

    /// Check current daily limit state
    pub fn check(&self, current_pnl: Decimal, trade_count: u32) -> DailyLimitState {
        // Check if we need to reset for new day
        let now = OffsetDateTime::now_utc();
        if now.date() != self.last_reset.date() {
            return DailyLimitState::Normal;
        }

        // Check loss limit
        let daily_loss = current_pnl.min(Decimal::ZERO).abs();
        let max_loss = self.current_equity * self.daily_loss_limit_percent;

        if daily_loss >= max_loss {
            return DailyLimitState::LimitReached;
        }

        // Check warning threshold
        if daily_loss >= max_loss * self.warning_threshold {
            let remaining = max_loss - daily_loss;
            let remaining_pct = remaining / max_loss;
            return DailyLimitState::NearLimit { remaining_pct };
        }

        // Check trade count
        if trade_count >= self.max_daily_trades {
            return DailyLimitState::LimitReached;
        }

        DailyLimitState::Normal
    }

    /// Update daily PnL
    pub fn update_pnl(&mut self, pnl: Decimal) {
        self.check_and_reset_day();
        self.today_pnl += pnl;
    }

    /// Record a new trade
    pub fn record_trade(&mut self) {
        self.check_and_reset_day();
        self.today_trades += 1;
    }

    /// Update current equity
    pub fn update_equity(&mut self, equity: Decimal) {
        self.current_equity = equity;
    }

    /// Get remaining daily loss capacity
    pub fn remaining_loss_capacity(&self, current_pnl: Decimal) -> Decimal {
        if self.current_equity <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let daily_loss = current_pnl.min(Decimal::ZERO).abs();
        let max_loss = self.current_equity * self.daily_loss_limit_percent;
        (max_loss - daily_loss).max(Decimal::ZERO)
    }

    /// Get remaining trade slots
    pub fn remaining_trades(&self, current_count: u32) -> u32 {
        self.max_daily_trades.saturating_sub(current_count)
    }

    /// Manual reset (e.g., for testing or edge cases)
    pub fn reset(&mut self) {
        self.today_pnl = Decimal::ZERO;
        self.today_trades = 0;
        self.last_reset = OffsetDateTime::now_utc();
    }

    fn check_and_reset_day(&mut self) {
        let now = OffsetDateTime::now_utc();
        if now.date() != self.last_reset.date() {
            self.today_pnl = Decimal::ZERO;
            self.today_trades = 0;
            self.last_reset = now;
        }
    }

    /// Get daily summary
    pub fn daily_summary(&self) -> DailySummary {
        DailySummary {
            pnl: self.today_pnl,
            trades: self.today_trades,
            loss_limit: self.current_equity * self.daily_loss_limit_percent,
            trade_limit: self.max_daily_trades,
            remaining_trades: self.max_daily_trades.saturating_sub(self.today_trades),
            is_limit_reached: self.check(self.today_pnl, self.today_trades) == DailyLimitState::LimitReached,
        }
    }
}

impl Default for DailyLimitsEngine {
    fn default() -> Self {
        Self::new(
            Decimal::from_f64(0.05).unwrap(), // 5% daily loss
            10,
        )
    }
}

/// Summary of daily trading activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    /// Current PnL
    pub pnl: Decimal,
    /// Number of trades taken
    pub trades: u32,
    /// Loss limit amount
    pub loss_limit: Decimal,
    /// Trade limit
    pub trade_limit: u32,
    /// Remaining trade slots
    pub remaining_trades: u32,
    /// Whether limit is reached
    pub is_limit_reached: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine() -> DailyLimitsEngine {
        DailyLimitsEngine::new(Decimal::from_f64(0.05).unwrap(), 10)
    }

    #[test]
    fn test_normal_state() {
        let engine = test_engine();
        let state = engine.check(Decimal::ZERO, 0);
        assert_eq!(state, DailyLimitState::Normal);
        assert!(!state.blocks_new_positions());
    }

    #[test]
    fn test_trade_limit() {
        let engine = test_engine();
        let state = engine.check(Decimal::ZERO, 10);
        assert_eq!(state, DailyLimitState::LimitReached);
        assert!(state.blocks_new_positions());
    }

    #[test]
    fn test_loss_scaling() {
        let state = DailyLimitState::NearLimit {
            remaining_pct: Decimal::from_f64(0.5).unwrap(),
        };
        assert_eq!(state.position_scaling(), Decimal::from_f64(0.5).unwrap());
    }

    #[test]
    fn test_remaining_capacity() {
        let mut engine = test_engine();
        engine.update_equity(Decimal::from(10000));

        let capacity = engine.remaining_loss_capacity(Decimal::from(-100));
        assert!(capacity > Decimal::ZERO);
        assert!(capacity <= Decimal::from(500)); // 5% of 10k = 500
    }
}
