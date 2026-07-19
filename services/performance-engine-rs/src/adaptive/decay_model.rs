use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecayModel {
    pub alpha: Decimal, // Smoothing factor
}

impl DecayModel {
    pub fn new(alpha: Decimal) -> Self {
        assert!(
            alpha > dec!(0.0) && alpha <= dec!(1.0),
            "Alpha must be between 0 and 1 exclusive"
        );
        Self { alpha }
    }

    /// Calculate the exponential moving average given the new value and the previous EMA.
    /// EMA_today = Value_today * alpha + EMA_yesterday * (1 - alpha)
    pub fn apply(&self, new_value: Decimal, prev_ema: Option<Decimal>) -> Decimal {
        match prev_ema {
            Some(prev) => {
                let one_minus_alpha = dec!(1.0) - self.alpha;
                (new_value * self.alpha) + (prev * one_minus_alpha)
            }
            None => new_value,
        }
    }
}
