use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct DepthScore;

impl DepthScore {
    pub fn calculate(available_depth: Decimal, required_depth: Decimal) -> Decimal {
        if required_depth <= Decimal::ZERO {
            return dec!(100);
        }
        let ratio = available_depth / required_depth;
        if ratio >= dec!(1) {
            dec!(100)
        } else {
            (ratio * dec!(100)).trunc_with_scale(2)
        }
    }
}
