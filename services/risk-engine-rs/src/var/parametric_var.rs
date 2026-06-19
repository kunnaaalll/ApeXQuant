use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use super::confidence_levels::ConfidenceLevel;

#[derive(Debug, Clone)]
pub struct ParametricVaR {
    count: u64,
    mean: Decimal,
    m2: Decimal, // For Welford's algorithm to compute variance
}

impl Default for ParametricVaR {
    fn default() -> Self {
        Self::new()
    }
}

impl ParametricVaR {
    pub fn new() -> Self {
        Self {
            count: 0,
            mean: Decimal::ZERO,
            m2: Decimal::ZERO,
        }
    }

    /// Update running mean and variance using Welford's online algorithm
    pub fn add_return(&mut self, ret: Decimal) {
        self.count += 1;
        let count_dec = Decimal::from(self.count);
        let delta = ret - self.mean;
        self.mean += delta / count_dec;
        let delta2 = ret - self.mean;
        self.m2 += delta * delta2;
    }

    pub fn variance(&self) -> Decimal {
        if self.count < 2 {
            Decimal::ZERO
        } else {
            self.m2 / Decimal::from(self.count - 1)
        }
    }

    pub fn standard_deviation(&self) -> Decimal {
        let var = self.variance();
        if var <= Decimal::ZERO {
            Decimal::ZERO
        } else {
            // rust_decimal has sqrt
            var.sqrt().unwrap_or(Decimal::ZERO)
        }
    }

    pub fn volatility_estimate(&self) -> Decimal {
        self.standard_deviation()
    }

    /// Compute parametric VaR for a given confidence level.
    /// VaR = -(mean - z * std_dev)
    /// Constrained to be >= 0.
    pub fn compute_var(&self, level: ConfidenceLevel) -> Decimal {
        let std_dev = self.standard_deviation();
        let z = level.z_score();
        
        // Parametric VaR assumes returns are normally distributed.
        // The return at the percentile is: mean - z * std_dev
        // The VaR is the negative of that return (representing loss).
        let worst_expected_return = self.mean - (z * std_dev);
        
        if worst_expected_return < Decimal::ZERO {
            -worst_expected_return
        } else {
            Decimal::ZERO
        }
    }

    pub fn var_90(&self) -> Decimal {
        self.compute_var(ConfidenceLevel::Ninety)
    }

    pub fn var_95(&self) -> Decimal {
        self.compute_var(ConfidenceLevel::NinetyFive)
    }

    pub fn var_99(&self) -> Decimal {
        self.compute_var(ConfidenceLevel::NinetyNine)
    }

    pub fn var_999(&self) -> Decimal {
        self.compute_var(ConfidenceLevel::NinetyNineNine)
    }
}
