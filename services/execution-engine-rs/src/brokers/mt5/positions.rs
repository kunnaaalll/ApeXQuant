use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::OpenPosition;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mt5Position {
    pub ticket: String,
    pub symbol: String,
    pub side: String,
    pub volume: Decimal,
    pub entry_price: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub floating_pnl: Decimal,
}

impl Into<OpenPosition> for Mt5Position {
    fn into(self) -> OpenPosition {
        OpenPosition {
            ticket: self.ticket,
            symbol: self.symbol,
            side: self.side,
            volume: self.volume,
            entry_price: self.entry_price,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
            floating_pnl: self.floating_pnl,
        }
    }
}
