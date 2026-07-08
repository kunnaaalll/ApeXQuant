pub use super::errors::BrokerError;
use super::health::BrokerHealth;
use super::requests::{
    ClosePositionRequest, OrderCancelRequest, OrderModifyRequest, OrderSubmitRequest,
};
use super::responses::{
    AccountInfo, ClosePositionResponse, OpenPosition, OrderCancelResponse, OrderModifyResponse,
    OrderSubmitResponse, PendingOrder, SymbolInfo,
};
use async_trait::async_trait;

pub type AccountState = AccountInfo;
pub type OrderState = PendingOrder;
pub type PositionState = OpenPosition;

#[async_trait]
pub trait BrokerAdapter: Send + Sync {
    async fn get_account(&self) -> Result<AccountInfo, BrokerError>;
    async fn get_symbol(&self, symbol: &str) -> Result<SymbolInfo, BrokerError>;
    async fn get_positions(&self) -> Result<Vec<OpenPosition>, BrokerError>;
    async fn get_orders(&self) -> Result<Vec<PendingOrder>, BrokerError>;

    async fn submit_order(
        &self,
        req: OrderSubmitRequest,
    ) -> Result<OrderSubmitResponse, BrokerError>;
    async fn modify_order(
        &self,
        req: OrderModifyRequest,
    ) -> Result<OrderModifyResponse, BrokerError>;
    async fn cancel_order(
        &self,
        req: OrderCancelRequest,
    ) -> Result<OrderCancelResponse, BrokerError>;
    async fn close_position(
        &self,
        req: ClosePositionRequest,
    ) -> Result<ClosePositionResponse, BrokerError>;

    async fn health(&self) -> Result<BrokerHealth, BrokerError>;
    async fn ping(&self) -> Result<(), BrokerError>;
}
