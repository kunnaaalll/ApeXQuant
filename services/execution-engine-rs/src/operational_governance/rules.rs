use crate::connection_supervisor::ConnectionState;
use crate::order_reconciliation::ReconciliationState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradingAuthorization {
    Approved,
    Denied(String),
}

#[derive(Debug, Clone)]
pub struct GovernanceState {
    pub connection_state: ConnectionState,
    pub reconciliation_state: ReconciliationState,
    pub has_account_mismatch: bool,
    pub has_position_parity: bool,
    pub risk_engine_approved: bool,
}

#[derive(Default)]
pub struct GovernanceEngine;

impl GovernanceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn authorize_trading(&self, state: &GovernanceState) -> TradingAuthorization {
        if !state.risk_engine_approved {
            return TradingAuthorization::Denied("Risk engine rejected trading".to_string());
        }

        if state.connection_state != ConnectionState::Healthy {
            return TradingAuthorization::Denied(format!(
                "Connection state is {:?}",
                state.connection_state
            ));
        }

        if let ReconciliationState::CriticalMismatch(_) = state.reconciliation_state {
            return TradingAuthorization::Denied("Critical order mismatch detected".to_string());
        }

        if state.has_account_mismatch {
            return TradingAuthorization::Denied("Account mismatch detected".to_string());
        }

        if !state.has_position_parity {
            return TradingAuthorization::Denied("Position parity not achieved".to_string());
        }

        TradingAuthorization::Approved
    }
}
