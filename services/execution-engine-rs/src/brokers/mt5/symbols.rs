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

impl From<Mt5Symbol> for SymbolInfo {
    fn from(val: Mt5Symbol) -> Self {
        SymbolInfo {
            symbol: val.symbol,
            digits: val.digits,
            lot_step: val.lot_step,
            tick_size: val.tick_size,
            minimum_volume: val.minimum_volume,
            maximum_volume: val.maximum_volume,
        }
    }
}
