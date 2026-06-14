use serde::{Deserialize, Serialize};
use crate::simulator::replay_engine::{ReplayEngine, ReplayFilter, ReplayResult, TradeRecord};

/// A configuration variant to test against historical data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub variant_id: String,
    pub description: String,
    pub filter: ReplayFilter,
}

/// The outcome of running a single variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantResult {
    pub variant_id: String,
    pub description: String,
    pub result: ReplayResult,
}

pub struct VariantRunner;

impl VariantRunner {
    /// Run all variants deterministically over the same historical trade set.
    /// Returns results in the same order as input variants.
    pub fn run_all(trades: &[TradeRecord], variants: &[Variant]) -> Vec<VariantResult> {
        variants
            .iter()
            .map(|v| VariantResult {
                variant_id: v.variant_id.clone(),
                description: v.description.clone(),
                result: ReplayEngine::replay(trades, &v.filter),
            })
            .collect()
    }
}
