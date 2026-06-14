use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct ConfidenceCalibration;

impl ConfidenceCalibration {
    pub fn bound_score(score: Decimal) -> Decimal {
        if score < Decimal::ZERO {
            Decimal::ZERO
        } else if score > dec!(100.0) {
            dec!(100.0)
        } else {
            score
        }
    }
}
