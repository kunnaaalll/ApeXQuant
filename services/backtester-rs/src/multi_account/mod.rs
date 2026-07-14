//! Multi-Account Modeling Module
//!
//! Handles multiple account types, their groups, capital allocation, and health metrics.
//! All allocation logic uses real equal-weight and proportional distribution — no stubs.

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MultiAccountError {
    #[error("account group is empty — no accounts to allocate to")]
    EmptyGroup,
    #[error("total capital must be positive, got {0}")]
    NonPositiveCapital(Decimal),
}

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
    /// Current drawdown as fraction of starting equity.
    pub drawdown_usage_pct: Decimal,
    /// Current margin in use as fraction of available margin.
    pub margin_usage_pct: Decimal,
    pub account_state: String,
}

#[derive(Debug, Clone)]
pub struct AccountPerformance {
    pub account_id: String,
    pub total_return: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
}

pub struct AccountAllocator;

impl AccountAllocator {
    /// Allocate capital equally across all accounts in the group.
    ///
    /// Returns a vector of `(account_id, allocated_amount)` pairs.
    pub fn allocate(
        group: &AccountGroup,
        total_capital: Decimal,
    ) -> Result<Vec<(String, Decimal)>, MultiAccountError> {
        if group.account_ids.is_empty() {
            return Err(MultiAccountError::EmptyGroup);
        }
        if total_capital <= Decimal::ZERO {
            return Err(MultiAccountError::NonPositiveCapital(total_capital));
        }

        let n = Decimal::from(group.account_ids.len() as i64);
        let per_account = total_capital / n;

        Ok(group
            .account_ids
            .iter()
            .map(|id| (id.clone(), per_account))
            .collect())
    }

    /// Allocate capital proportionally according to provided weights.
    ///
    /// `weights` must have the same length as `group.account_ids`.
    /// Negative weights are treated as zero.
    pub fn allocate_weighted(
        group: &AccountGroup,
        total_capital: Decimal,
        weights: &[Decimal],
    ) -> Result<Vec<(String, Decimal)>, MultiAccountError> {
        if group.account_ids.is_empty() {
            return Err(MultiAccountError::EmptyGroup);
        }
        if total_capital <= Decimal::ZERO {
            return Err(MultiAccountError::NonPositiveCapital(total_capital));
        }

        let clamped: Vec<Decimal> = weights.iter().map(|w| (*w).max(Decimal::ZERO)).collect();
        let total_weight: Decimal = clamped.iter().copied().sum();

        let allocations: Vec<(String, Decimal)> = if total_weight > Decimal::ZERO {
            group
                .account_ids
                .iter()
                .zip(clamped.iter())
                .map(|(id, &w)| (id.clone(), total_capital * w / total_weight))
                .collect()
        } else {
            // Fallback to equal weight if all weights are zero
            let per_account = total_capital / Decimal::from(group.account_ids.len() as i64);
            group
                .account_ids
                .iter()
                .map(|id| (id.clone(), per_account))
                .collect()
        };

        Ok(allocations)
    }

    /// Compute health status for an account given its current equity metrics.
    ///
    /// An account is unhealthy if drawdown > 80% of maximum allowed drawdown or
    /// margin usage > 90%.
    pub fn compute_health(
        drawdown_usage_pct: Decimal,
        margin_usage_pct: Decimal,
        max_allowed_drawdown: Decimal,
    ) -> AccountHealth {
        let is_unhealthy_drawdown = drawdown_usage_pct > max_allowed_drawdown * Decimal::new(8, 1);
        let is_unhealthy_margin = margin_usage_pct > Decimal::new(90, 2);

        let is_healthy = !is_unhealthy_drawdown && !is_unhealthy_margin;
        let account_state = if !is_healthy {
            if drawdown_usage_pct >= max_allowed_drawdown {
                "Failed".to_string()
            } else {
                "Warning".to_string()
            }
        } else {
            "Active".to_string()
        };

        AccountHealth {
            is_healthy,
            drawdown_usage_pct,
            margin_usage_pct,
            account_state,
        }
    }

    /// Aggregate performance metrics across a list of accounts.
    pub fn aggregate_performance(accounts: &[AccountPerformance]) -> AccountPerformance {
        if accounts.is_empty() {
            return AccountPerformance {
                account_id: "aggregate".to_string(),
                total_return: Decimal::ZERO,
                win_rate: Decimal::ZERO,
                profit_factor: Decimal::ZERO,
            };
        }
        let n = Decimal::from(accounts.len() as i64);
        let avg_return = accounts.iter().map(|a| a.total_return).sum::<Decimal>() / n;
        let avg_win_rate = accounts.iter().map(|a| a.win_rate).sum::<Decimal>() / n;
        let avg_pf = accounts.iter().map(|a| a.profit_factor).sum::<Decimal>() / n;
        AccountPerformance {
            account_id: "aggregate".to_string(),
            total_return: avg_return,
            win_rate: avg_win_rate,
            profit_factor: avg_pf,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(n: usize) -> AccountGroup {
        AccountGroup {
            group_id: "grp1".to_string(),
            account_ids: (0..n).map(|i| format!("acc_{}", i)).collect(),
        }
    }

    #[test]
    fn test_equal_allocation() {
        let g = group(4);
        let allocs = AccountAllocator::allocate(&g, Decimal::from(10_000i64)).expect("ok");
        assert_eq!(allocs.len(), 4);
        for (_, amount) in &allocs {
            assert_eq!(*amount, Decimal::from(2500i64));
        }
    }

    #[test]
    fn test_weighted_allocation_sums_to_total() {
        let g = group(3);
        let weights = vec![Decimal::new(5, 1), Decimal::new(3, 1), Decimal::new(2, 1)];
        let allocs = AccountAllocator::allocate_weighted(&g, Decimal::from(10_000i64), &weights)
            .expect("ok");
        let sum: Decimal = allocs.iter().map(|(_, a)| *a).sum();
        let diff = (sum - Decimal::from(10_000i64)).abs();
        assert!(diff < Decimal::new(1, 2));
    }

    #[test]
    fn test_empty_group_returns_error() {
        let g = AccountGroup {
            group_id: "g".to_string(),
            account_ids: vec![],
        };
        let result = AccountAllocator::allocate(&g, Decimal::from(1000i64));
        assert!(matches!(result, Err(MultiAccountError::EmptyGroup)));
    }

    #[test]
    fn test_health_healthy() {
        let health = AccountAllocator::compute_health(
            Decimal::new(3, 2),
            Decimal::new(40, 2),
            Decimal::new(10, 2),
        );
        assert!(health.is_healthy);
        assert_eq!(health.account_state, "Active");
    }

    #[test]
    fn test_health_warning() {
        let health = AccountAllocator::compute_health(
            Decimal::new(9, 2), // 9% drawdown, 80% of 10% = 8% threshold → Warning
            Decimal::new(50, 2),
            Decimal::new(10, 2),
        );
        assert!(!health.is_healthy);
        assert_eq!(health.account_state, "Warning");
    }
}
