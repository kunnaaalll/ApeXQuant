//! Prop Firm Constraints Module
//!
//! Models proprietary trading firm rules: daily drawdown limits, max total drawdown,
//! profit targets, minimum trading days, and consistency rules.
//!
//! All state transitions are computed from real equity data — no hardcoded outcomes.

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConstraintError {
    #[error("starting equity must be positive, got {0}")]
    NonPositiveStartingEquity(Decimal),
    #[error("highest equity cannot be less than starting equity")]
    InvalidHighestEquity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountState {
    /// All constraints satisfied, account is operating normally.
    Active,
    /// Approaching a limit (daily drawdown > 80% of allowed, or equity near minimum).
    Warning,
    /// A soft limit was breached — trading may be restricted but account is not failed.
    Restricted,
    /// A hard limit was breached — account is failed / blown.
    Failed,
    /// Profit target reached AND all other requirements satisfied.
    Passed,
}

#[derive(Debug, Clone)]
pub struct PropConstraints {
    /// Maximum allowed intraday drawdown as a fraction of starting equity (e.g. 0.05 = 5%).
    pub max_daily_drawdown: Decimal,
    /// Maximum allowed total drawdown as a fraction of starting equity (e.g. 0.10 = 10%).
    pub max_total_drawdown: Decimal,
    /// Profit target as a fraction of starting equity (e.g. 0.10 = 10%). None = no target.
    pub profit_target: Option<Decimal>,
    /// Minimum number of trading days required before payout qualification.
    pub min_trading_days: u32,
    /// Consistency rule: largest single-day profit must not exceed this fraction of total profit.
    /// None = no consistency rule applied.
    pub consistency_rule_pct: Option<Decimal>,
}

impl PropConstraints {
    /// Standard FTMO Phase 1 constraints.
    pub fn ftmo_phase1() -> Self {
        Self {
            max_daily_drawdown: Decimal::new(5, 2),   // 5%
            max_total_drawdown: Decimal::new(10, 2),  // 10%
            profit_target: Some(Decimal::new(10, 2)), // 10%
            min_trading_days: 4,
            consistency_rule_pct: None,
        }
    }

    /// Standard FTMO Phase 2 constraints.
    pub fn ftmo_phase2() -> Self {
        Self {
            max_daily_drawdown: Decimal::new(5, 2),
            max_total_drawdown: Decimal::new(10, 2),
            profit_target: Some(Decimal::new(5, 2)), // 5%
            min_trading_days: 4,
            consistency_rule_pct: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintEvaluator;

impl ConstraintEvaluator {
    /// Evaluate all prop firm constraints against current account metrics.
    ///
    /// Parameters:
    /// - `constraints`: the applicable rule set.
    /// - `current_equity`: current account equity.
    /// - `starting_equity`: equity at the start of the evaluation period.
    /// - `highest_equity`: the highest equity reached during the period.
    /// - `lowest_daily_equity`: the lowest equity reached intraday today.
    /// - `trading_days`: number of days on which at least one trade was executed.
    /// - `best_day_profit`: the single-day profit from the best trading day.
    /// - `total_profit`: cumulative profit for the period.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate(
        constraints: &PropConstraints,
        current_equity: Decimal,
        starting_equity: Decimal,
        highest_equity: Decimal,
        lowest_daily_equity: Decimal,
        trading_days: u32,
        best_day_profit: Decimal,
        total_profit: Decimal,
    ) -> Result<AccountState, ConstraintError> {
        if starting_equity <= Decimal::ZERO {
            return Err(ConstraintError::NonPositiveStartingEquity(starting_equity));
        }

        // ── Hard limits: evaluate first ──────────────────────────────────────

        // Max total drawdown: (starting_equity - current_equity) / starting_equity
        let total_dd = (starting_equity - current_equity) / starting_equity;
        if total_dd >= constraints.max_total_drawdown {
            return Ok(AccountState::Failed);
        }

        // Daily drawdown: (highest intraday equity - lowest intraday equity) / starting_equity
        let daily_dd = if highest_equity > lowest_daily_equity {
            (highest_equity - lowest_daily_equity) / starting_equity
        } else {
            Decimal::ZERO
        };
        if daily_dd >= constraints.max_daily_drawdown {
            return Ok(AccountState::Failed);
        }

        // ── Consistency rule ─────────────────────────────────────────────────
        if let Some(consistency_pct) = constraints.consistency_rule_pct {
            if total_profit > Decimal::ZERO && best_day_profit > Decimal::ZERO {
                let best_day_fraction = best_day_profit / total_profit;
                if best_day_fraction > consistency_pct {
                    // Best day profit exceeds consistency threshold — Restricted
                    return Ok(AccountState::Restricted);
                }
            }
        }

        // ── Passed: all targets met ──────────────────────────────────────────
        if let Some(target) = constraints.profit_target {
            let profit_fraction = (current_equity - starting_equity) / starting_equity;
            if profit_fraction >= target && trading_days >= constraints.min_trading_days {
                return Ok(AccountState::Passed);
            }
        }

        // ── Warning: approaching daily limit ────────────────────────────────
        let warning_threshold = constraints.max_daily_drawdown * Decimal::new(8, 1); // 80%
        if daily_dd >= warning_threshold {
            return Ok(AccountState::Warning);
        }

        // ── Warning: approaching total drawdown limit ────────────────────────
        let total_warning = constraints.max_total_drawdown * Decimal::new(8, 1);
        if total_dd >= total_warning {
            return Ok(AccountState::Warning);
        }

        Ok(AccountState::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_when_within_all_limits() {
        let c = PropConstraints::ftmo_phase1();
        let state = ConstraintEvaluator::evaluate(
            &c,
            Decimal::from(10_200i64), // $200 profit on $10k
            Decimal::from(10_000i64),
            Decimal::from(10_300i64),
            Decimal::from(10_100i64),
            2,
            Decimal::from(200i64),
            Decimal::from(200i64),
        )
        .expect("ok");
        assert_eq!(state, AccountState::Active);
    }

    #[test]
    fn test_failed_on_daily_drawdown_breach() {
        let c = PropConstraints::ftmo_phase1(); // max daily = 5%
        let state = ConstraintEvaluator::evaluate(
            &c,
            Decimal::from(9_400i64),
            Decimal::from(10_000i64),
            Decimal::from(10_000i64),
            Decimal::from(9_400i64), // 6% daily DD
            1,
            Decimal::ZERO,
            Decimal::ZERO,
        )
        .expect("ok");
        assert_eq!(state, AccountState::Failed);
    }

    #[test]
    fn test_failed_on_total_drawdown_breach() {
        let c = PropConstraints::ftmo_phase1(); // max total = 10%
        let state = ConstraintEvaluator::evaluate(
            &c,
            Decimal::from(8_900i64), // 11% total DD
            Decimal::from(10_000i64),
            Decimal::from(10_000i64),
            Decimal::from(8_900i64),
            1,
            Decimal::ZERO,
            Decimal::ZERO,
        )
        .expect("ok");
        assert_eq!(state, AccountState::Failed);
    }

    #[test]
    fn test_passed_when_target_reached() {
        let c = PropConstraints::ftmo_phase1(); // target = 10%, min days = 4
        let state = ConstraintEvaluator::evaluate(
            &c,
            Decimal::from(11_000i64), // 10% profit
            Decimal::from(10_000i64),
            Decimal::from(11_000i64),
            Decimal::from(10_800i64),
            4, // meets minimum days
            Decimal::from(200i64),
            Decimal::from(1_000i64),
        )
        .expect("ok");
        assert_eq!(state, AccountState::Passed);
    }

    #[test]
    fn test_warning_near_daily_limit() {
        let c = PropConstraints::ftmo_phase1(); // max daily = 5%, warning at 4%
        let state = ConstraintEvaluator::evaluate(
            &c,
            Decimal::from(9_650i64),
            Decimal::from(10_000i64),
            Decimal::from(10_000i64),
            Decimal::from(9_600i64), // 4% daily DD → warning
            1,
            Decimal::ZERO,
            Decimal::ZERO,
        )
        .expect("ok");
        assert_eq!(state, AccountState::Warning);
    }
}
