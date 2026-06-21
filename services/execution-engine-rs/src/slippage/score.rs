use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct SlippageScore;

impl SlippageScore {
    pub fn calculate(realized_slippage: Decimal, max_acceptable_slippage: Decimal) -> Decimal {
        if max_acceptable_slippage <= Decimal::ZERO {
            return if realized_slippage <= Decimal::ZERO { dec!(100) } else { dec!(0) };
        }
        
        if realized_slippage <= Decimal::ZERO {
            return dec!(100);
        }
        
        let ratio = realized_slippage / max_acceptable_slippage;
        if ratio >= dec!(1) {
            return dec!(0);
        }
        
        ((dec!(1) - ratio) * dec!(100)).trunc_with_scale(2)
    }
}
