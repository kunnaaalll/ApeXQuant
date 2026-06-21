use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo {
    pub balance: Decimal,
    pub equity: Decimal,
    pub free_margin: Decimal,
    pub leverage: Decimal,
    pub margin_level: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolInfo {
    pub symbol: String,
    pub digits: u32,
    pub lot_step: Decimal,
    pub tick_size: Decimal,
    pub minimum_volume: Decimal,
    pub maximum_volume: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenPosition {
    pub ticket: String,
    pub symbol: String,
    pub side: String,
    pub volume: Decimal,
    pub entry_price: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub floating_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingOrder {
    pub ticket: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub volume: Decimal,
    pub price: Decimal,
    pub status: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderSubmitResponse {
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderModifyResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderCancelResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClosePositionResponse {
    pub success: bool,
}
