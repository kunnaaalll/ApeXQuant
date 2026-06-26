//! Prop Firm Constraints Module
//!
//! Models proprietary trading firm rules such as daily drawdowns, max drawdowns,
//! consistency rules, and evaluates account states based on these constraints.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountState {
    Active,
    Warning,
    Restricted,
    Failed,
    Passed,
}

#[derive(Debug, Clone)]
pub struct PropConstraints {
    pub max_daily_drawdown: Decimal,
    pub max_total_drawdown: Decimal,
    pub profit_target: Option<Decimal>,
    pub min_trading_days: u32,
    pub consistency_rule_pct: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct ConstraintEvaluator;

impl ConstraintEvaluator {
    pub fn evaluate(
        _constraints: &PropConstraints,
        _current_equity: Decimal,
        _starting_equity: Decimal,
        _highest_equity: Decimal,
        _trading_days: u32,
    ) -> Result<AccountState, &'static str> {
        // Stub: evaluate constraints against current account metrics
        Ok(AccountState::Active)
    }
}
