use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterVariant {
    pub variant_id: String,
    pub sl: Decimal,
    pub tp: Decimal,
    pub rr: Decimal,
    pub filter_score: Decimal,
    pub entry_quality: Decimal,
    pub outcome: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterComparisonResult {
    pub best_variant: ParameterVariant,
    pub worst_variant: ParameterVariant,
    pub difference: Decimal,
    pub confidence: Decimal,
}

pub struct ParameterComparisonEngine;

impl ParameterComparisonEngine {
    pub fn compare(
        mut variants: Vec<ParameterVariant>,
        confidence: Decimal,
    ) -> Option<ParameterComparisonResult> {
        if variants.len() < 2 {
            return None;
        }

        // Sort by outcome descending
        variants.sort_by(|a, b| b.outcome.cmp(&a.outcome));

        let best_variant = variants.first().unwrap().clone();
        let worst_variant = variants.last().unwrap().clone();
        let difference = best_variant.outcome - worst_variant.outcome;

        Some(ParameterComparisonResult {
            best_variant,
            worst_variant,
            difference,
            confidence,
        })
    }
}
