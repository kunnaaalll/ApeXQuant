use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::OpenPosition;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinancePosition {
    pub symbol: String,
    pub position_side: String,
    pub position_amount: Decimal,
    pub entry_price: Decimal,
    pub unrealized_profit: Decimal,
    pub leverage: Decimal,
    pub mark_price: Decimal,
}

impl From<BinancePosition> for OpenPosition {
    fn from(val: BinancePosition) -> Self {
        OpenPosition {
            ticket: format!("{}-{}", val.symbol, val.position_side),
            symbol: val.symbol,
            side: val.position_side,
            volume: val.position_amount.abs(),
            entry_price: val.entry_price,
            stop_loss: None,
            take_profit: None,
            floating_pnl: val.unrealized_profit,
        }
    }
}
