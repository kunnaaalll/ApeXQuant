use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct SpreadCost;

impl SpreadCost {
    pub fn calculate(ask: Decimal, bid: Decimal) -> Decimal {
        if ask < bid {
            return Decimal::ZERO;
        }
        (ask - bid) / dec!(2)
    }
}
