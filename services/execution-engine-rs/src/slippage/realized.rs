use rust_decimal::Decimal;

pub struct RealizedSlippage;

impl RealizedSlippage {
    pub fn calculate(expected_price: Decimal, execution_price: Decimal, is_buy: bool) -> Decimal {
        if is_buy {
            execution_price - expected_price
        } else {
            expected_price - execution_price
        }
    }
}
