use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct FixedFractionalSizer {
    pub risk_fraction: Decimal, // e.g. 0.02 for 2%
}

impl FixedFractionalSizer {
    pub fn new(risk_fraction: Decimal) -> Self {
        Self { risk_fraction }
    }

    /// Calculate size: (equity * risk_fraction) / stop_loss_distance
    pub fn calculate_size(&self, equity: Decimal, stop_loss_distance: Decimal) -> Decimal {
        if stop_loss_distance <= Decimal::ZERO || equity <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let risk_amount = equity * self.risk_fraction;
        (risk_amount / stop_loss_distance).trunc_with_scale(4)
    }
}
