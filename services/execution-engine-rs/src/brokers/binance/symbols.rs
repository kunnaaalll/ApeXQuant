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

impl From<BinanceSymbol> for SymbolInfo {
    fn from(val: BinanceSymbol) -> Self {
        SymbolInfo {
            symbol: val.symbol,
            digits: val.price_precision,
            lot_step: val.step_size,
            tick_size: val.tick_size,
            minimum_volume: val.min_qty,
            maximum_volume: val.max_qty,
        }
    }
}
