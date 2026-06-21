use rust_decimal::Decimal;

pub struct MarketImpact;

impl MarketImpact {
    pub fn calculate(order_size: Decimal, average_daily_volume: Decimal) -> Decimal {
        if average_daily_volume == Decimal::ZERO {
            return Decimal::MAX;
        }
        // Linear ratio used to avoid floating point math (no sqrt)
        (order_size / average_daily_volume).trunc_with_scale(8)
    }
}
