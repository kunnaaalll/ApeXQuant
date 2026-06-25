use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationCommand {
    pub account_id: String,
    pub action: String,
    pub payload: String,
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
        trade_payload: &str,
    ) -> Vec<OrchestrationCommand> {
        let mut commands = Vec::new();

        for (account_id, multiplier) in slave_accounts {
            if *multiplier > Decimal::ZERO {
                commands.push(OrchestrationCommand {
                    account_id: account_id.clone(),
                    action: "EXECUTE_TRADE".to_string(),
                    payload: format!("{}_MULTIPLIED_BY_{}", trade_payload, multiplier),
                });
            }
        }

        commands
    }
}
