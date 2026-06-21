use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct AvailabilityScore;

impl AvailabilityScore {
    pub fn calculate(uptime_ratio: Decimal) -> Decimal {
        if uptime_ratio >= dec!(1) {
            return dec!(100);
        } else if uptime_ratio <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        (uptime_ratio * dec!(100)).trunc_with_scale(2)
    }
}
