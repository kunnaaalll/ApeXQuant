use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAllocation {
    pub account_id: String,
    pub allocated_capital: Decimal,
}

pub struct AccountAllocatorEngine;

impl Default for AccountAllocatorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountAllocatorEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn allocate_capital(
        &self,
        total_capital: Decimal,
        account_weights: &[(String, Decimal)],
    ) -> Vec<AccountAllocation> {
        let mut allocations = Vec::new();
        if account_weights.is_empty() || total_capital <= Decimal::ZERO {
            return allocations;
        }

        let total_weight: Decimal = account_weights.iter().map(|(_, w)| *w).sum();
        if total_weight == Decimal::ZERO {
            return allocations; // Cannot allocate if total weight is zero
        }

        let mut allocated = Decimal::ZERO;

        for (account_id, weight) in account_weights {
            let normalized_weight = weight / total_weight;
            let amount = (total_capital * normalized_weight).round_dp(2);
            allocations.push(AccountAllocation {
                account_id: account_id.clone(),
                allocated_capital: amount,
            });
            allocated += amount;
        }

        // Handle remainder if any (due to rounding), assign to the first account
        if allocated != total_capital {
            allocations[0].allocated_capital += total_capital - allocated;
        }

        allocations
    }
}
