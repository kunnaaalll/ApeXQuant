use rust_decimal::Decimal;

pub struct ExpectedSlippage;

impl ExpectedSlippage {
    pub fn calculate(volatility: Decimal, order_size: Decimal, liquidity_depth: Decimal) -> Decimal {
        if liquidity_depth == Decimal::ZERO {
            return Decimal::MAX;
        }
        // Simple formula: volatility * (order_size / liquidity_depth)
        (volatility * (order_size / liquidity_depth)).trunc_with_scale(8)
    }
}
