use rust_decimal::Decimal;
use std::cmp;

#[derive(Debug, Clone)]
pub struct DriftEngine;

impl Default for DriftEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn measure_absolute_difference(&self, a: Decimal, b: Decimal) -> Decimal {
        (a - b).abs()
    }

    pub fn measure_relative_difference(&self, a: Decimal, b: Decimal, reference: Decimal) -> Decimal {
        let difference = self.measure_absolute_difference(a, b);
        let max_ref = cmp::max(reference.abs(), Decimal::ONE);
        
        let percentage_difference = (difference / max_ref) * Decimal::new(100, 0);
        
        let zero = Decimal::ZERO;
        let hundred = Decimal::new(100, 0);
        
        cmp::min(cmp::max(percentage_difference, zero), hundred)
    }
}
