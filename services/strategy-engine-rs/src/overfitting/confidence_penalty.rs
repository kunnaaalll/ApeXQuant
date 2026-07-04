use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct OverfitConfidencePenalty;

impl OverfitConfidencePenalty {
    pub fn calculate_penalty(overfit_ratio: Decimal) -> Decimal {
        if overfit_ratio > dec!(2.0) {
            dec!(0.5) // Deduct 50% confidence
        } else if overfit_ratio > dec!(1.5) {
            dec!(0.2) // Deduct 20% confidence
        } else {
            Decimal::ZERO
        }
    }
}
