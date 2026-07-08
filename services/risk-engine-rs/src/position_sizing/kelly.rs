use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct KellySizer {
    pub max_risk_fraction: Decimal, // e.g. 0.05 for 5% max risk cap
}

impl KellySizer {
    pub fn new(max_risk_fraction: Decimal) -> Self {
        Self { max_risk_fraction }
    }

    /// Calculate Kelly fraction: f = w - (1 - w) / R
    /// where w is win probability, R is win/loss ratio
    pub fn calculate_fraction(&self, win_probability: Decimal, win_loss_ratio: Decimal) -> Decimal {
        if win_loss_ratio <= Decimal::ZERO || win_probability <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let one = dec!(1.0);
        let kelly = win_probability - (one - win_probability) / win_loss_ratio;
        kelly.max(Decimal::ZERO).min(self.max_risk_fraction)
    }

    /// Calculate size: (equity * kelly_fraction) / stop_loss_distance
    pub fn calculate_size(&self, equity: Decimal, stop_loss_distance: Decimal, win_probability: Decimal, win_loss_ratio: Decimal) -> Decimal {
        if stop_loss_distance <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        let fraction = self.calculate_fraction(win_probability, win_loss_ratio);
        let risk_amount = equity * fraction;
        (risk_amount / stop_loss_distance).trunc_with_scale(4)
    }
}
