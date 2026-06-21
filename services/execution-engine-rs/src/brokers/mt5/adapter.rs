use crate::brokers::broker::BrokerAdapter;
use crate::brokers::errors::BrokerError;
use crate::brokers::health::BrokerHealth;
use crate::brokers::requests::{ClosePositionRequest, OrderCancelRequest, OrderModifyRequest, OrderSubmitRequest};
use crate::brokers::responses::{AccountInfo, ClosePositionResponse, OpenPosition, OrderCancelResponse, OrderModifyResponse, OrderSubmitResponse, PendingOrder, SymbolInfo};

use async_trait::async_trait;

pub struct Mt5Adapter {
    pub broker_id: String,
}

impl Mt5Adapter {
    pub fn new(broker_id: String) -> Self {
        Self { broker_id }
    }
}

#[async_trait]
impl BrokerAdapter for Mt5Adapter {
    async fn get_account(&self) -> Result<AccountInfo, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn get_symbol(&self, _symbol: &str) -> Result<SymbolInfo, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn get_positions(&self) -> Result<Vec<OpenPosition>, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn get_orders(&self) -> Result<Vec<PendingOrder>, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn submit_order(&self, _req: OrderSubmitRequest) -> Result<OrderSubmitResponse, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn modify_order(&self, _req: OrderModifyRequest) -> Result<OrderModifyResponse, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn cancel_order(&self, _req: OrderCancelRequest) -> Result<OrderCancelResponse, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn close_position(&self, _req: ClosePositionRequest) -> Result<ClosePositionResponse, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn health(&self) -> Result<BrokerHealth, BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }

    async fn ping(&self) -> Result<(), BrokerError> {
        Err(BrokerError::InternalError("Not implemented yet".to_string()))
    }
}

