use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::AccountInfo;

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

impl Into<AccountInfo> for BinanceAccount {
    fn into(self) -> AccountInfo {
        use rust_decimal_macros::dec;
        let margin_level = if self.maintenance_margin > dec!(0.0) {
            (self.margin_balance / self.maintenance_margin) * dec!(100.0)
        } else {
            dec!(0.0)
        };

        AccountInfo {
            balance: self.wallet_balance,
            equity: self.margin_balance,
            free_margin: self.available_balance,
            leverage: self.leverage,
            margin_level,
        }
    }
}
