use crate::simulator::replay_engine::TradeRecord;
use crate::simulator::variant_runner::{Variant, VariantResult, VariantRunner};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Evaluation of a full configuration sweep against historical data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationEvaluation {
    pub best_variant_id: String,
    pub worst_variant_id: String,
    pub best_expectancy: Decimal,
    pub worst_expectancy: Decimal,
    pub difference: Decimal,
    pub all_results: Vec<VariantResult>,
    pub explanation: String,
}

pub struct ConfigurationEvaluator;

impl ConfigurationEvaluator {
    /// Evaluates all configuration variants and identifies best/worst.
    /// Deterministic: equal expectancies are tiebroken by variant_id (lexicographic).
    pub fn evaluate(
        trades: &[TradeRecord],
        variants: Vec<Variant>,
    ) -> Option<ConfigurationEvaluation> {
        if variants.is_empty() {
            return None;
        }

        let results = VariantRunner::run_all(trades, &variants);

        let mut sorted = results.clone();
        sorted.sort_by(|a, b| {
            b.result
                .expectancy
                .cmp(&a.result.expectancy)
                .then(a.variant_id.cmp(&b.variant_id))
        });

        let best = sorted.first()?;
        let worst = sorted.last()?;

        let difference = best.result.expectancy - worst.result.expectancy;

        let explanation = format!(
            "Best: {} (expectancy {:.3}R) | Worst: {} (expectancy {:.3}R) | Spread: {:.3}R",
            best.variant_id,
            best.result.expectancy,
            worst.variant_id,
            worst.result.expectancy,
            difference,
        );

        Some(ConfigurationEvaluation {
            best_variant_id: best.variant_id.clone(),
            worst_variant_id: worst.variant_id.clone(),
            best_expectancy: best.result.expectancy,
            worst_expectancy: worst.result.expectancy,
            difference,
            all_results: results,
            explanation,
        })
    }
}
