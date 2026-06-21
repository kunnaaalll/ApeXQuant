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

impl Into<OpenPosition> for BinancePosition {
    fn into(self) -> OpenPosition {
        OpenPosition {
            ticket: format!("{}-{}", self.symbol, self.position_side),
            symbol: self.symbol,
            side: self.position_side,
            volume: self.position_amount.abs(),
            entry_price: self.entry_price,
            stop_loss: None,
            take_profit: None,
            floating_pnl: self.unrealized_profit,
        }
    }
}
