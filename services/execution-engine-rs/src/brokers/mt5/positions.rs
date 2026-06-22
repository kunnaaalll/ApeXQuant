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

impl From<Mt5Position> for OpenPosition {
    fn from(val: Mt5Position) -> Self {
        OpenPosition {
            ticket: val.ticket,
            symbol: val.symbol,
            side: val.side,
            volume: val.volume,
            entry_price: val.entry_price,
            stop_loss: val.stop_loss,
            take_profit: val.take_profit,
            floating_pnl: val.floating_pnl,
        }
    }
}
