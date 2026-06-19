use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecayModel {
    alpha: Decimal,
}

impl DecayModel {
    pub fn new(alpha: Decimal) -> Self {
        let alpha = Self::clamp_alpha(alpha);
        Self { alpha }
    }

    pub fn update(&self, previous: Decimal, current: Decimal) -> Decimal {
        let one = dec!(1.0);
        let current_weighted = current * self.alpha;
        let previous_weighted = previous * (one - self.alpha);
        current_weighted + previous_weighted
    }

    fn clamp_alpha(alpha: Decimal) -> Decimal {
        let min = dec!(0.01);
        let max = dec!(0.50);
        if alpha < min {
            min
        } else if alpha > max {
            max
        } else {
            alpha
        }
    }
}
