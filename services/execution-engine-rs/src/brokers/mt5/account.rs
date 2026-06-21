use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::AccountInfo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mt5Account {
    pub balance: Decimal,
    pub equity: Decimal,
    pub free_margin: Decimal,
    pub leverage: Decimal,
    pub margin_level: Decimal,
}

impl Into<AccountInfo> for Mt5Account {
    fn into(self) -> AccountInfo {
        AccountInfo {
            balance: self.balance,
            equity: self.equity,
            free_margin: self.free_margin,
            leverage: self.leverage,
            margin_level: self.margin_level,
        }
    }
}
