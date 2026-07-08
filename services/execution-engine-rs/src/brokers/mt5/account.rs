use crate::brokers::responses::AccountInfo;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mt5Account {
    pub balance: Decimal,
    pub equity: Decimal,
    pub free_margin: Decimal,
    pub leverage: Decimal,
    pub margin_level: Decimal,
}

impl From<Mt5Account> for AccountInfo {
    fn from(val: Mt5Account) -> Self {
        AccountInfo {
            balance: val.balance,
            equity: val.equity,
            free_margin: val.free_margin,
            leverage: val.leverage,
            margin_level: val.margin_level,
        }
    }
}
