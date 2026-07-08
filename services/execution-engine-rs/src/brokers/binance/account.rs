use crate::brokers::responses::AccountInfo;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinanceAccount {
    pub wallet_balance: Decimal,
    pub unrealized_pnl: Decimal,
    pub margin_balance: Decimal,
    pub maintenance_margin: Decimal,
    pub initial_margin: Decimal,
    pub available_balance: Decimal,
    pub leverage: Decimal,
}

impl From<BinanceAccount> for AccountInfo {
    fn from(val: BinanceAccount) -> Self {
        use rust_decimal_macros::dec;
        let margin_level = if val.maintenance_margin > dec!(0.0) {
            (val.margin_balance / val.maintenance_margin) * dec!(100.0)
        } else {
            dec!(0.0)
        };

        AccountInfo {
            balance: val.wallet_balance,
            equity: val.margin_balance,
            free_margin: val.available_balance,
            leverage: val.leverage,
            margin_level,
        }
    }
}
