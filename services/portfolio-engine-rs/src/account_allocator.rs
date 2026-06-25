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
        let mut allocated = Decimal::ZERO;

        for (account_id, weight) in account_weights {
            let amount = total_capital * weight;
            allocations.push(AccountAllocation {
                account_id: account_id.clone(),
                allocated_capital: amount,
            });
            allocated += amount;
        }
        
        // Handle remainder if any (due to precision), could be assigned to first account
        if allocated < total_capital && !allocations.is_empty() {
            allocations[0].allocated_capital += total_capital - allocated;
        }

        allocations
    }
}
