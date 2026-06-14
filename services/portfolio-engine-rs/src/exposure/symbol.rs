use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SymbolExposure {
    pub symbol_id: String, // String representation like "EURUSD"
    pub position_count: usize,
    pub total_size: Decimal, // Could represent base asset size
    pub average_entry: Decimal,
    pub risk_contribution: Decimal,
    pub current_pnl: Decimal,
    pub weight: Decimal,
}

impl SymbolExposure {
    pub fn new(symbol_id: String) -> Self {
        Self {
            symbol_id,
            position_count: 0,
            total_size: Decimal::ZERO,
            average_entry: Decimal::ZERO,
            risk_contribution: Decimal::ZERO,
            current_pnl: Decimal::ZERO,
            weight: Decimal::ZERO,
        }
    }
}
