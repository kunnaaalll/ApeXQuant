//! Holding Period Analyzer
//!
//! Computes PnL efficiency metrics for an open/closed position.
//! Uses `rust_decimal::Decimal` throughout — no f32 or f64 business logic.

use rust_decimal::Decimal;

pub struct HoldingPeriodAnalyzer;

impl HoldingPeriodAnalyzer {
    /// PnL per hour of holding — positive = efficient use of time.
    ///
    /// Returns `Decimal::ZERO` if holding_duration_secs ≤ 0 to avoid division by zero.
    pub fn pnl_per_hour(realized_pnl: Decimal, holding_duration_secs: i64) -> Decimal {
        if holding_duration_secs <= 0 {
            return Decimal::ZERO;
        }
        let hours = Decimal::from(holding_duration_secs) / Decimal::from(3600);
        if hours == Decimal::ZERO {
            return Decimal::ZERO;
        }
        realized_pnl / hours
    }

    /// Holding efficiency: normalised PnL-per-hour relative to account balance.
    ///
    /// Efficiency = (pnl_per_hour / account_balance) × 100
    /// Represents annualised hourly return contribution (%).
    pub fn holding_efficiency(
        realized_pnl: Decimal,
        holding_duration_secs: i64,
        account_balance: Decimal,
    ) -> Decimal {
        if account_balance <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        let pph = Self::pnl_per_hour(realized_pnl, holding_duration_secs);
        (pph / account_balance * Decimal::from(100)).round_dp(4)
    }

    /// Momentum: rate of change of unrealized PnL over the holding period.
    ///
    /// Uses a simple two-point approximation: (current_pnl - previous_pnl) / duration_secs.
    /// Positive = accelerating winner; negative = decelerating.
    pub fn pnl_momentum(
        current_unrealized_pnl: Decimal,
        previous_unrealized_pnl: Decimal,
        interval_secs: i64,
    ) -> Decimal {
        if interval_secs <= 0 {
            return Decimal::ZERO;
        }
        (current_unrealized_pnl - previous_unrealized_pnl) / Decimal::from(interval_secs)
    }

    /// Realised PnL for a closed trade (long or short).
    ///
    /// Long:  (exit_price - entry_price) × quantity
    /// Short: (entry_price - exit_price) × quantity
    /// Net of commission deducted from gross PnL.
    pub fn realized_pnl(
        entry_price: Decimal,
        exit_price: Decimal,
        quantity: Decimal,
        is_long: bool,
        commission: Decimal,
    ) -> Decimal {
        let gross = if is_long {
            (exit_price - entry_price) * quantity
        } else {
            (entry_price - exit_price) * quantity
        };
        gross - commission
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pnl_per_hour() {
        // 100 PnL over 2 hours = 50/hour
        let result = HoldingPeriodAnalyzer::pnl_per_hour(
            Decimal::new(100, 0),
            7200, // 2 hours
        );
        assert_eq!(result, Decimal::new(50, 0));
    }

    #[test]
    fn test_pnl_per_hour_zero_duration() {
        let result = HoldingPeriodAnalyzer::pnl_per_hour(Decimal::new(100, 0), 0);
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_realized_pnl_long() {
        // Buy 100 units at 1.1000, exit at 1.1050, commission 0
        let pnl = HoldingPeriodAnalyzer::realized_pnl(
            Decimal::new(11000, 4),
            Decimal::new(11050, 4),
            Decimal::new(100, 0),
            true,
            Decimal::ZERO,
        );
        // (1.1050 - 1.1000) × 100 = 0.0050 × 100 = 0.50
        assert_eq!(pnl, Decimal::new(50, 2));
    }

    #[test]
    fn test_realized_pnl_short() {
        // Short 100 units at 1.1050, cover at 1.1000
        let pnl = HoldingPeriodAnalyzer::realized_pnl(
            Decimal::new(11050, 4),
            Decimal::new(11000, 4),
            Decimal::new(100, 0),
            false,
            Decimal::ZERO,
        );
        assert_eq!(pnl, Decimal::new(50, 2));
    }

    #[test]
    fn test_holding_efficiency_normalised() {
        // 100 PnL over 1 hour on 10,000 balance = (100/1) / 10,000 × 100 = 1%
        let eff = HoldingPeriodAnalyzer::holding_efficiency(
            Decimal::new(100, 0),
            3600,
            Decimal::new(10_000, 0),
        );
        assert_eq!(eff, Decimal::new(1, 0));
    }
}
