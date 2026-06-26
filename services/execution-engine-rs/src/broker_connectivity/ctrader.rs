use super::{BrokerAdapter, BrokerError, AccountState, OrderState, PositionState};
use async_trait::async_trait;

#[derive(Debug)]
pub struct CTraderAdapter {
    connected: bool,
    logged_in: bool,
}

impl CTraderAdapter {
    pub fn new() -> Self {
        Self {
            connected: false,
            logged_in: false,
        }
    }
}

#[async_trait]
impl BrokerAdapter for CTraderAdapter {
    async fn login(&mut self) -> Result<(), BrokerError> {
        self.connected = true;
        self.logged_in = true;
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<(), BrokerError> {
        self.connected = true;
        self.logged_in = true;
        Ok(())
    }

    async fn heartbeat(&self) -> Result<(), BrokerError> {
        if self.connected {
            Ok(())
        } else {
            Err(BrokerError::ConnectionLost)
        }
    }
    
    async fn sync_account(&self) -> Result<AccountState, BrokerError> {
        if !self.logged_in {
            return Err(BrokerError::AuthenticationFailed);
        }
        Ok(AccountState {
            balance: 10000.0,
            equity: 10000.0,
            margin: 0.0,
            free_margin: 10000.0,
            leverage: 100.0,
            drawdown: 0.0,
        })
    }

    async fn sync_orders(&self) -> Result<Vec<OrderState>, BrokerError> {
        if !self.logged_in {
            return Err(BrokerError::AuthenticationFailed);
        }
        Ok(vec![])
    }

    async fn sync_positions(&self) -> Result<Vec<PositionState>, BrokerError> {
        if !self.logged_in {
            return Err(BrokerError::AuthenticationFailed);
        }
        Ok(vec![])
    }
}
