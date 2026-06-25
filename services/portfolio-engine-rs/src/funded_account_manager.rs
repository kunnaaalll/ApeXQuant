use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountPhase {
    Evaluation1,
    Evaluation2,
    Funded,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundedAccountState {
    pub account_id: String,
    pub phase: AccountPhase,
    pub starting_balance: Decimal,
    pub current_equity: Decimal,
    pub high_water_mark: Decimal,
}

pub struct FundedAccountManager;

impl Default for FundedAccountManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FundedAccountManager {
    pub fn new() -> Self {
        Self
    }

    pub fn check_status(&self, state: &FundedAccountState, profit_target: Decimal, max_drawdown: Decimal) -> AccountPhase {
        if state.phase == AccountPhase::Terminated {
            return AccountPhase::Terminated;
        }

        let drawdown = state.high_water_mark - state.current_equity;
        if drawdown >= max_drawdown {
            return AccountPhase::Terminated;
        }

        let profit = state.current_equity - state.starting_balance;
        if profit >= profit_target && state.phase == AccountPhase::Evaluation1 {
            return AccountPhase::Evaluation2;
        }
        
        if profit >= profit_target && state.phase == AccountPhase::Evaluation2 {
            return AccountPhase::Funded;
        }

        state.phase.clone()
    }
}
