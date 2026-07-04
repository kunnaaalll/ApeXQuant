use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Pure function to calculate raw Kelly fraction
pub fn calculate_kelly_fraction(win_probability: Decimal, win_loss_ratio: Decimal) -> Decimal {
    if win_loss_ratio <= Decimal::ZERO || win_probability <= Decimal::ZERO {
        return Decimal::ZERO;
    }

    let one = dec!(1.0);
    let fraction = win_probability - (one - win_probability) / win_loss_ratio;
    fraction.max(Decimal::ZERO)
}
