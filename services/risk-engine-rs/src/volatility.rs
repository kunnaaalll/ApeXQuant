use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct VolatilityModel {
    pub decay_factor: Decimal, // lambda for EWMA, e.g. 0.94
}

impl VolatilityModel {
    pub fn new(decay_factor: Decimal) -> Self {
        Self { decay_factor }
    }

    /// Calculate EWMA volatility from returns
    pub fn calculate_ewma(&self, returns: &[Decimal], previous_variance: Decimal) -> Decimal {
        if returns.is_empty() {
            return previous_variance;
        }

        let mut variance = previous_variance;
        let lambda = self.decay_factor;
        let one_minus_lambda = dec!(1.0) - lambda;

        for &ret in returns {
            let ret_sq = ret * ret;
            variance = lambda * variance + one_minus_lambda * ret_sq;
        }

        variance
    }

    /// Calculate standard deviation (realized volatility)
    pub fn calculate_realized_volatility(returns: &[Decimal]) -> Decimal {
        if returns.len() < 2 {
            return Decimal::ZERO;
        }

        let count = Decimal::from(returns.len());
        let sum: Decimal = returns.iter().sum();
        let mean = sum / count;

        let variance_sum: Decimal = returns.iter()
            .map(|&ret| {
                let diff = ret - mean;
                diff * diff
            })
            .sum();

        let variance = variance_sum / (count - dec!(1.0));
        use rust_decimal::MathematicalOps;
        variance.sqrt().unwrap_or(Decimal::ZERO)
    }
}
