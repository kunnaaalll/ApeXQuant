use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::integrations::execution::ExecutionOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationCommand {
    pub account_id: String,
    pub action: String,
    pub order: ExecutionOrder,
}

pub struct MultiAccountOrchestrator;

impl Default for MultiAccountOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiAccountOrchestrator {
    pub fn new() -> Self {
        Self
    }

    pub fn sync_trade(
        &self,
        _master_account_id: &str,
        slave_accounts: &[(String, Decimal)], // Account ID and Multiplier
        master_order: &ExecutionOrder,
    ) -> Vec<OrchestrationCommand> {
        let mut commands = Vec::new();

        for (account_id, multiplier) in slave_accounts {
            if *multiplier > Decimal::ZERO {
                let mut slave_order = master_order.clone();
                slave_order.quantity = master_order.quantity * multiplier;
                
                commands.push(OrchestrationCommand {
                    account_id: account_id.clone(),
                    action: "EXECUTE_TRADE".to_string(),
                    order: slave_order,
                });
            }
        }

        commands
    }
}
