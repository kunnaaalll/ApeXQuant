//! Multi-Account Modeling Module
//!
//! Handles multiple account types, their groups, allocation logic, and health metrics.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountType {
    Demo,
    Evaluation,
    Funded,
    Personal,
    Research,
}

#[derive(Debug, Clone)]
pub struct AccountGroup {
    pub group_id: String,
    pub account_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AccountHealth {
    pub is_healthy: bool,
    pub drawdown_usage_pct: Decimal,
    pub margin_usage_pct: Decimal,
}

#[derive(Debug, Clone)]
pub struct AccountPerformance {
    pub total_return: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
}

pub struct AccountAllocator;

impl AccountAllocator {
    pub fn allocate(_group: &AccountGroup, _total_capital: Decimal) -> Result<Vec<(String, Decimal)>, &'static str> {
        // Stub: Allocate capital across accounts in the group
        Ok(vec![])
    }
}
