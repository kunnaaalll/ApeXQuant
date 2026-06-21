use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::SymbolInfo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinanceSymbol {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub price_precision: u32,
    pub quantity_precision: u32,
    pub step_size: Decimal,
    pub tick_size: Decimal,
    pub min_qty: Decimal,
    pub max_qty: Decimal,
}

impl Into<SymbolInfo> for BinanceSymbol {
    fn into(self) -> SymbolInfo {
        SymbolInfo {
            symbol: self.symbol,
            digits: self.price_precision,
            lot_step: self.step_size,
            tick_size: self.tick_size,
            minimum_volume: self.min_qty,
            maximum_volume: self.max_qty,
        }
    }
}
