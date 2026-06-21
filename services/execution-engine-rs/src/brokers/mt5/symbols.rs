use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::SymbolInfo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mt5Symbol {
    pub symbol: String,
    pub digits: u32,
    pub lot_step: Decimal,
    pub tick_size: Decimal,
    pub minimum_volume: Decimal,
    pub maximum_volume: Decimal,
}

impl Into<SymbolInfo> for Mt5Symbol {
    fn into(self) -> SymbolInfo {
        SymbolInfo {
            symbol: self.symbol,
            digits: self.digits,
            lot_step: self.lot_step,
            tick_size: self.tick_size,
            minimum_volume: self.minimum_volume,
            maximum_volume: self.maximum_volume,
        }
    }
}
