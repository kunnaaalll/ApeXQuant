use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOrder {
    pub symbol: String,
    pub quantity: Decimal,
    pub is_buy: bool,
    pub order_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub order_id: String,
    pub status: String,
    pub filled_quantity: Decimal,
    pub average_price: Decimal,
}

pub struct ExecutionClient;

impl Default for ExecutionClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn submit_order(&self, _order: &ExecutionOrder) -> Result<ExecutionResult, String> {
        // Placeholder for submitting an order to execution engine
        Ok(ExecutionResult {
            order_id: "ORD-1234".to_string(),
            status: "FILLED".to_string(),
            filled_quantity: Decimal::ZERO,
            average_price: Decimal::ZERO,
        })
    }
}
