use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct SpreadQuality;

impl SpreadQuality {
    pub fn calculate(current_spread: Decimal, historical_average_spread: Decimal) -> Decimal {
        if historical_average_spread <= Decimal::ZERO {
            return if current_spread <= Decimal::ZERO {
                dec!(100)
            } else {
                dec!(0)
            };
        }
        if current_spread <= Decimal::ZERO {
            return dec!(100);
        }
        let ratio = current_spread / historical_average_spread;
        if ratio <= dec!(1) {
            dec!(100)
        } else if ratio >= dec!(2) {
            Decimal::ZERO
        } else {
            ((dec!(2) - ratio) * dec!(100)).trunc_with_scale(2)
        }
    }
}
