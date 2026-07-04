use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

pub struct StrategyEvolutionEngine;

impl StrategyEvolutionEngine {
    /// Evolve parameters: if expectancy improves, keep mutation.
    /// Simple perturbation mutation logic.
    pub fn evolve_parameters(
        parameters: &HashMap<String, Decimal>,
        expectancy_improved: bool,
    ) -> HashMap<String, Decimal> {
        let mut evolved = parameters.clone();
        if expectancy_improved {
            // Keep parameters or make a small positive perturbation
            for (_, val) in evolved.iter_mut() {
                *val = (*val * dec!(1.01)).trunc_with_scale(4);
            }
        } else {
            // Mutate/perturb parameters in alternative direction
            for (_, val) in evolved.iter_mut() {
                *val = (*val * dec!(0.99)).trunc_with_scale(4);
            }
        }
        evolved
    }
}
