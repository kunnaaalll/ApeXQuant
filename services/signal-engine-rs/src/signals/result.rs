use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalDirection {
    Long,
    Short,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    pub signal_id: String,
    pub symbol: String,
    pub direction: SignalDirection,
    pub confidence: f64,
    pub confluence_score: f64,
    pub entry_price: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub patterns: Vec<String>,
    pub regime: String,
    pub timestamp: time::OffsetDateTime,
}
