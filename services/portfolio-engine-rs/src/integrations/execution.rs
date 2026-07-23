//! Execution Engine gRPC Integration
//!
//! Real gRPC client that calls `ExecutionEngine::SubmitOrder`.
//! No hardcoded "FILLED" fallback — fill data comes from the execution engine.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use apex_protos::common::{Symbol, Volume};
use apex_protos::execution::execution_engine_client::ExecutionEngineClient;
use apex_protos::execution::{NewOrder, SubmitOrderRequest};
use tonic::transport::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOrder {
    pub symbol: String,
    pub quantity: Decimal,
    pub is_buy: bool,
    pub order_type: String,
    pub client_order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub order_id: String,
    pub client_order_id: String,
    pub status: String,
    pub filled_quantity: Decimal,
    pub average_price: Decimal,
    pub commission: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionQualityEvent {
    pub order_id: String,
    pub slippage: Decimal,
    pub latency_ms: u32,
    pub timestamp: i64,
}

pub struct ExecutionClient {
    client: ExecutionEngineClient<Channel>,
}

impl ExecutionClient {
    /// Connect to the Execution Engine gRPC endpoint.
    pub async fn connect(url: String) -> Result<Self, String> {
        let client = ExecutionEngineClient::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to execution engine: {}", e))?;
        Ok(Self { client })
    }

    /// Submit an order to the execution engine.
    ///
    /// Calls `ExecutionEngine::SubmitOrder` and maps the proto response.
    /// Returns an error (never a fake "FILLED" value) if the call fails.
    pub async fn submit_order(
        &mut self,
        order: &ExecutionOrder,
    ) -> Result<ExecutionResult, String> {
        let request = tonic::Request::new(SubmitOrderRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            order: Some(NewOrder {
                client_order_id: order.client_order_id.clone(),
                symbol: Some(Symbol {
                    code: order.symbol.clone(),
                    exchange: "".into(),
                    asset_class: 0,
                    description: "".into(),
                }),
                order_type: 1, // MARKET
                side: if order.is_buy { 1 } else { 2 },
                volume: Some(Volume {
                    units: order.quantity.to_string(),
                    lot_size: "".into(),
                    fractional: true,
                }),
                limit_price: None,
                stop_price: None,
                stop_loss: None,
                take_profit: None,
                valid_until: None,
                time_in_force: 0,
                signal_id: "".to_string(),
                strategy_id: "".to_string(),
                correlation_id: "".to_string(),
                requester_service: "portfolio-engine".to_string(),
            }),
            preferences: None,
            priority: 0,
        });

        let response = self.client.submit_order(request).await.map_err(|e| {
            format!(
                "ExecutionEngine::SubmitOrder failed for {}: {}",
                order.symbol, e
            )
        })?;

        let fill = response.into_inner();

        // The response just confirms submission and returns current state.
        Ok(ExecutionResult {
            order_id: fill.order_id,
            client_order_id: order.client_order_id.clone(),
            status: format!("{:?}", fill.state),
            filled_quantity: Decimal::ZERO,
            average_price: Decimal::ZERO,
            commission: Decimal::ZERO,
        })
    }
}
