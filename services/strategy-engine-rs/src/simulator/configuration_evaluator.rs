use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct ConfigurationEvaluator;

impl ConfigurationEvaluator {
    /// Evaluate simulation run: returns combined performance score.
    pub fn evaluate_run(metrics: &HashMap<String, Decimal>) -> Decimal {
        let pf = metrics
            .get("profit_factor")
            .copied()
            .unwrap_or(Decimal::ZERO);
        let win_rate = metrics.get("win_rate").copied().unwrap_or(Decimal::ZERO);
        pf * win_rate
    }
}
